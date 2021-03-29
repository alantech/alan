use std::error::Error;

use futures::future::join_all;
use futures::stream::StreamExt;
use heim_common::units::{information::kilobyte, time::second};
#[cfg(target_os = "linux")]
use heim_cpu::os::linux::CpuTimeExt;
#[cfg(target_os = "linux")]
use heim_memory::os::linux::MemoryExt;
use log::error;
use serde::Serialize;
use tokio::time::{sleep, Duration};

#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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

// calculate the cpu times usage per process using the process'
// total cpu time delta in a given time window
async fn get_proc_usages() -> Vec<f64> {
  let duration = 1.0;
  let futures = heim_process::processes()
    .map(|process| async {
      match process {
        Ok(proc) => {
          let cpu_1 = proc.cpu_time().await;
          sleep(Duration::from_secs(duration as u64)).await;
          let cpu_2 = proc.cpu_time().await;
          // account for zombie process
          if cpu_1.is_err() || cpu_2.is_err() {
            return 0.0;
          }
          let times_1 = cpu_1.unwrap();
          let times_2 = cpu_2.unwrap();
          let system = times_2.system().get::<second>() - times_1.system().get::<second>();
          let user = times_2.user().get::<second>() - times_1.user().get::<second>();
          (user + system) / duration
        }
        Err(_) => 0.0,
      }
    })
    .collect::<Vec<_>>()
    .await;
  join_all(futures).await
}

// get total cpu times per core since the VM's uptime
// while setting linux specific fields to 0
#[cfg(not(target_os = "linux"))]
async fn get_cores_total_times() -> Vec<CPUSecsV1> {
  heim_cpu::times()
    .map(|r| {
      if let Ok(cpu) = r {
        CPUSecsV1 {
          user: cpu.user().get::<second>(),
          system: cpu.system().get::<second>(),
          idle: cpu.idle().get::<second>(),
          irq: 0.0,
          nice: 0.0,
          ioWait: 0.0,
          softIrq: 0.0,
          steal: 0.0,
        }
      } else {
        CPUSecsV1 {
          user: 0.0,
          system: 0.0,
          idle: 0.0,
          irq: 0.0,
          nice: 0.0,
          ioWait: 0.0,
          softIrq: 0.0,
          steal: 0.0,
        }
      }
    })
    .collect()
    .await
}

// get total cpu times per core since the VM's uptime
#[cfg(target_os = "linux")]
async fn get_cores_total_times() -> Vec<CPUSecsV1> {
  heim_cpu::times()
    .map(|r| {
      if let Ok(cpu) = r {
        CPUSecsV1 {
          user: cpu.user().get::<second>(),
          system: cpu.system().get::<second>(),
          idle: cpu.idle().get::<second>(),
          irq: cpu.irq().get::<second>(),
          nice: cpu.nice().get::<second>(),
          ioWait: cpu.io_wait().get::<second>(),
          softIrq: cpu.soft_irq().get::<second>(),
          steal: cpu.steal().get::<second>(),
        }
      } else {
        CPUSecsV1 {
          user: 0.0,
          system: 0.0,
          idle: 0.0,
          irq: 0.0,
          nice: 0.0,
          ioWait: 0.0,
          softIrq: 0.0,
          steal: 0.0,
        }
      }
    })
    .collect()
    .await
}

// Cpu Times from /proc/stat are for the entire lifetime of the VM
// so generate it twice with a wait in between to generate a time window
async fn get_cores_times() -> Vec<CPUSecsV1> {
  let times_1 = get_cores_total_times().await;
  sleep(Duration::from_secs(1)).await;
  let times_2 = get_cores_total_times().await;
  let mut time_window = Vec::new();
  for (idx, t2) in times_2.iter().enumerate() {
    let t1 = &times_1[idx];
    time_window.push(CPUSecsV1 {
      user: t2.user - t1.user,
      system: t2.system - t1.system,
      idle: t2.idle - t1.idle,
      irq: t2.irq - t1.irq,
      nice: t2.nice - t1.nice,
      ioWait: t2.ioWait - t1.ioWait,
      softIrq: t2.softIrq - t1.softIrq,
      steal: t2.steal - t1.steal,
    })
  }
  time_window
}

#[cfg(target_os = "linux")]
pub async fn get_v1_stats() -> Result<VMStatsV1, Box<dyn Error + Send + Sync>> {
  let memory = heim_memory::memory().await;
  match memory {
    Ok(memory) => {
      let swap = heim_memory::swap().await;
      match swap {
        Ok(swap) => Ok(VMStatsV1 {
          cpuSecs: get_cores_times().await,
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
        Err(_) => return Err("Failed to get swap information".into()),
      }
    }
    Err(_) => return Err("Failed to get system memory information".into()),
  }
}

// zero out linux specific stats
#[cfg(not(target_os = "linux"))]
pub async fn get_v1_stats() -> Result<VMStatsV1, Box<dyn Error + Send + Sync>> {
  let memory = heim_memory::memory().await;
  match memory {
    Ok(memory) => {
      let swap = heim_memory::swap().await;
      match swap {
        Ok(swap) => Ok(VMStatsV1 {
          cpuSecs: get_cores_times().await,
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
        Err(_) => return Err("Failed to get swap information".into()),
      }
    }
    Err(_) => return Err("Failed to get system memory information".into()),
  }
}
