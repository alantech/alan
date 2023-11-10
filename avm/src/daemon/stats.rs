use sysinfo::{ProcessExt, System, SystemExt};
use serde::Serialize;

use crate::daemon::daemon::DaemonResult;

#[derive(Debug, Clone, Serialize)]
struct CPUSample {
  avg_cpu_core_util: f64,
  max_proc_cpu_usage: f64,
}

#[derive(Debug, Clone, Serialize)]
struct CPURates {
  avg_cpu_core_util: &'static f64,
  max_proc_cpu_usage: &'static f64,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
pub struct CPUSecsV1 {
  user: f64,
  system: f64,
  idle: f64,
  irq: f64,
  nice: f64,
  ioWait: f64,
  softIrq: f64,
  steal: f64,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
pub struct VMStatsV1 {
  cpuSecs: Vec<CPUSecsV1>,
  procsCpuUsage: Vec<f64>,
  totalMemoryKb: u64,
  availableMemoryKb: u64,
  freeMemoryKb: u64,
  usedMemoryKb: u64,
  activeMemoryKb: u64,
  totalSwapKb: u64,
  usedSwapKb: u64,
  freeSwapKb: u64,
}

// calculate the cpu % usage per process using the process'
// total cpu time delta in a 100ms time window
async fn get_proc_usages(sys: &System) -> Vec<f64> {
  sys.processes().values().map(|process| {
    process.cpu_usage() as f64
  }).collect()
}

// Cpu Times from /proc/stat are for the entire lifetime of the VM
// so generate it twice with a wait in between to generate a time window
async fn get_cores_times(sys: &System) -> DaemonResult<Vec<CPUSecsV1>> {
  // TODO: Revive or replace this
  let mut time_window = Vec::new();
  for cpu in sys.cpus() {
    time_window.push(CPUSecsV1 {
      user: 0.0,
      system: 0.0,
      idle: 0.0,
      irq: 0.0,
      nice: 0.0,
      ioWait: 0.0,
      softIrq: 0.0,
      steal: 0.0,
    });
  }
  Ok(time_window)
}

pub async fn get_v1_stats() -> DaemonResult<VMStatsV1> {
  let mut sys = System::new_all();
  sys.refresh_all();
  let core_times = get_cores_times(&sys).await?;
  Ok(VMStatsV1 {
    cpuSecs: core_times,
    procsCpuUsage: get_proc_usages(&sys).await,
    totalMemoryKb: sys.total_memory(),
    availableMemoryKb: sys.available_memory(),
    freeMemoryKb: sys.free_memory(),
    activeMemoryKb: sys.used_memory(), // TODO: Revive this?
    usedMemoryKb: sys.used_memory(),
    totalSwapKb: sys.total_swap(),
    usedSwapKb: sys.used_swap(),
    freeSwapKb: sys.free_swap(),
  })
}

// returns the suggested scaling factor for the cluster
// 2 prescribes doubling the cluster size
// 1 prescribes leaving the cluster as-is
// 0.5 prescribes halving the cluster size
pub fn get_stats_factor(stats: &Vec<VMStatsV1>) -> String {
  let samples = get_cpu_procs_samples(stats).unwrap_or(Vec::new());
  // take avg of samples
  let avg_util = get_avg_cpu_util(&samples);
  let avg_max_proc_cpu_usage = get_avg_max_proc_cpu_usage(&samples);
  // single threaded processes like node and python are bounced around cores and
  // they use cpu time across all cores that are available so we use the average
  // cpu utilization across all cores which also works for multithreaded processes
  if avg_util < 0.3 && avg_max_proc_cpu_usage < 0.3 {
    return String::from("0.5");
  } else if avg_util > 0.8 || avg_max_proc_cpu_usage > 0.8 {
    return String::from("2");
  } else {
    return String::from("1");
  }
}

fn get_cpu_procs_samples(stats: &Vec<VMStatsV1>) -> DaemonResult<Vec<CPUSample>> {
  Ok(
    stats
      .iter()
      .map(|s| CPUSample {
        avg_cpu_core_util: get_avg_cpu_core_util(s),
        max_proc_cpu_usage: get_max_procs_usage(s),
      })
      .collect(),
  )
}

fn get_avg_cpu_core_util(stat: &VMStatsV1) -> f64 {
  let acc = stat
    .cpuSecs
    .iter()
    .map(|t| {
      let idle = t.idle + t.ioWait;
      // TODO ignore t.steal for now, but once we have basic clustering
      // we want to remove nodes with a high steal cpu time
      let total = total(t) - t.steal;
      let active = total - idle;
      return active / total;
    })
    .reduce(|a, b| a + b);
  match acc {
    Some(acc) => acc,
    None => (0 as f64),
  }
}

fn total(cpu_secs: &CPUSecsV1) -> f64 {
  return cpu_secs.user
    + cpu_secs.system
    + cpu_secs.softIrq
    + cpu_secs.irq
    + cpu_secs.softIrq
    + cpu_secs.nice
    + cpu_secs.ioWait
    + cpu_secs.idle;
}

fn get_max_procs_usage(stat: &VMStatsV1) -> f64 {
  let mut sorted_procs_cpu_usage = stat.procsCpuUsage.clone();
  sorted_procs_cpu_usage.sort_by(|a, b| a.partial_cmp(b).unwrap());
  match sorted_procs_cpu_usage.last() {
    Some(value) => *value,
    None => 0 as f64,
  }
}

fn get_avg_cpu_util(samples: &Vec<CPUSample>) -> f64 {
  match samples
    .iter()
    .map(|s| s.avg_cpu_core_util)
    .reduce(|a, b| a + b)
  {
    Some(acc) => acc / samples.len() as f64,
    None => 0 as f64,
  }
}

fn get_avg_max_proc_cpu_usage(samples: &Vec<CPUSample>) -> f64 {
  match samples
    .iter()
    .map(|s| s.max_proc_cpu_usage)
    .reduce(|a, b| a + b)
  {
    Some(acc) => acc / samples.len() as f64,
    None => 0 as f64,
  }
}
