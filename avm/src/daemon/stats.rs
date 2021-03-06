use futures::stream::StreamExt;
use heim_common::units::{information::kilobyte, time::second};
#[cfg(target_os = "linux")]
use heim_cpu::os::linux::CpuTimeExt;
#[cfg(target_os = "linux")]
use heim_memory::os::linux::MemoryExt;
use serde::Serialize;
use tokio::time::{sleep, Duration};

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
#[cfg(not(target_arch = "aarch64"))]
async fn get_proc_usages() -> Vec<f64> {
  use futures::future::join_all;
  use heim_common::units::time::millisecond;
  let duration = 100.0; // ms
  let futures = heim_process::processes()
    .map(|process| async {
      match process {
        Ok(proc) => {
          let cpu_1 = proc.cpu_time().await;
          sleep(Duration::from_millis(duration as u64)).await;
          let cpu_2 = proc.cpu_time().await;
          // account for zombie process
          if cpu_1.is_err() || cpu_2.is_err() {
            return 0.0;
          }
          let times_1 = cpu_1.unwrap();
          let times_2 = cpu_2.unwrap();
          let system =
            times_2.system().get::<millisecond>() - times_1.system().get::<millisecond>();
          let user = times_2.user().get::<millisecond>() - times_1.user().get::<millisecond>();
          (user + system) / duration
        }
        Err(_) => 0.0,
      }
    })
    .collect::<Vec<_>>()
    .await;
  join_all(futures).await
}
#[cfg(target_arch = "aarch64")]
async fn get_proc_usages() -> Vec<f64> {
  vec![]
}

// get total cpu times per core since the VM's uptime
// while setting linux specific fields to 0
#[cfg(not(target_os = "linux"))]
async fn get_cores_total_times() -> Vec<DaemonResult<CPUSecsV1>> {
  heim_cpu::times()
    .map(|r| {
      if let Ok(cpu) = r {
        Ok(CPUSecsV1 {
          user: cpu.user().get::<second>(),
          system: cpu.system().get::<second>(),
          idle: cpu.idle().get::<second>(),
          irq: 0.0,
          nice: 0.0,
          ioWait: 0.0,
          softIrq: 0.0,
          steal: 0.0,
        })
      } else {
        Err("Failed to get CPU times".into())
      }
    })
    .collect()
    .await
}

// get total cpu times per core since the VM's uptime
#[cfg(target_os = "linux")]
async fn get_cores_total_times() -> Vec<DaemonResult<CPUSecsV1>> {
  heim_cpu::times()
    .map(|r| {
      if let Ok(cpu) = r {
        Ok(CPUSecsV1 {
          user: cpu.user().get::<second>(),
          system: cpu.system().get::<second>(),
          idle: cpu.idle().get::<second>(),
          irq: cpu.irq().get::<second>(),
          nice: cpu.nice().get::<second>(),
          ioWait: cpu.io_wait().get::<second>(),
          softIrq: cpu.soft_irq().get::<second>(),
          steal: cpu.steal().get::<second>(),
        })
      } else if let Err(err_cpu) = r {
        Err(format!("Failed to get CPU times. {:?}", err_cpu).into())
      } else {
        Err("Failed to get CPU times".into())
      }
    })
    .collect()
    .await
}

// Cpu Times from /proc/stat are for the entire lifetime of the VM
// so generate it twice with a wait in between to generate a time window
async fn get_cores_times() -> DaemonResult<Vec<CPUSecsV1>> {
  let times_1 = get_cores_total_times().await;
  sleep(Duration::from_millis(100)).await;
  let times_2 = get_cores_total_times().await;
  let mut time_window = Vec::new();
  for (idx, t2) in times_2.iter().enumerate() {
    let t1 = &times_1[idx];
    if let (Ok(t1), Ok(t2)) = (t1, t2) {
      time_window.push(CPUSecsV1 {
        user: t2.user - t1.user,
        system: t2.system - t1.system,
        idle: t2.idle - t1.idle,
        irq: t2.irq - t1.irq,
        nice: t2.nice - t1.nice,
        ioWait: t2.ioWait - t1.ioWait,
        softIrq: t2.softIrq - t1.softIrq,
        steal: t2.steal - t1.steal,
      });
    } else if let Err(err_t1) = t1 {
      return Err(format!("Failed to get CPU times. {:?}", err_t1).into());
    } else if let Err(err_t2) = t2 {
      return Err(format!("Failed to get CPU times. {:?}", err_t2).into());
    } else {
      return Err("Failed to get CPU times".into());
    }
  }
  Ok(time_window)
}

#[cfg(target_os = "linux")]
pub async fn get_v1_stats() -> DaemonResult<VMStatsV1> {
  let memory = heim_memory::memory().await;
  let swap = heim_memory::swap().await;
  let core_times = get_cores_times().await?;
  match (memory, swap) {
    (Ok(memory), Ok(swap)) => Ok(VMStatsV1 {
      cpuSecs: core_times,
      procsCpuUsage: get_proc_usages().await,
      totalMemoryKb: memory.total().get::<kilobyte>(),
      availableMemoryKb: memory.available().get::<kilobyte>(),
      freeMemoryKb: memory.free().get::<kilobyte>(),
      activeMemoryKb: memory.active().get::<kilobyte>(),
      usedMemoryKb: memory.used().get::<kilobyte>(),
      totalSwapKb: swap.total().get::<kilobyte>(),
      usedSwapKb: swap.used().get::<kilobyte>(),
      freeSwapKb: swap.free().get::<kilobyte>(),
    }),
    (Err(err_memory), _) => {
      return Err(format!("Failed to get system memory information. {:?}", err_memory).into())
    }
    (_, Err(err_swap)) => {
      return Err(format!("Failed to get swap information. {:?}", err_swap).into())
    }
  }
}

// zero out linux specific stats
#[cfg(not(target_os = "linux"))]
pub async fn get_v1_stats() -> DaemonResult<VMStatsV1> {
  let memory = heim_memory::memory().await;
  let swap = heim_memory::swap().await;
  let core_times = get_cores_times().await?;
  match (memory, swap) {
    (Ok(memory), Ok(swap)) => Ok(VMStatsV1 {
      cpuSecs: core_times,
      procsCpuUsage: get_proc_usages().await,
      totalMemoryKb: memory.total().get::<kilobyte>(),
      availableMemoryKb: memory.available().get::<kilobyte>(),
      freeMemoryKb: memory.free().get::<kilobyte>(),
      activeMemoryKb: 0,
      usedMemoryKb: 0,
      totalSwapKb: swap.total().get::<kilobyte>(),
      usedSwapKb: swap.used().get::<kilobyte>(),
      freeSwapKb: swap.free().get::<kilobyte>(),
    }),
    (Err(_err_memory), _) => return Err("Failed to get system memory information".into()),
    (_, Err(_err_swap)) => return Err("Failed to get swap information".into()),
  }
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
