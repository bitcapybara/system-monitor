use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use futures::channel::oneshot;
use parking_lot::RwLock;
use sysinfo::{CpuExt, DiskExt, NetworkExt, SystemExt};

#[derive(Debug, Default)]
pub struct Cpu {
    pub name: String,
    pub usage: f32,
}

#[derive(Debug, Default)]
pub struct Memory {
    pub total: u64,
    pub available: u64,
    pub free: u64,
    pub used: u64,
    pub swap: Swap,
}

#[derive(Debug, Default)]
pub struct Swap {
    pub total: u64,
    pub free: u64,
    pub used: u64,
}

#[derive(Debug, Default)]
pub struct Network {
    pub name: String,
    pub received: u64,
    pub total_received: u64,
    pub transmitted: u64,
    pub total_transmitted: u64,
    pub packets_received: u64,
    pub total_packets_received: u64,
    pub packets_transmitted: u64,
    pub total_packets_transmitted: u64,
    pub errors_on_received: u64,
    pub total_errors_on_received: u64,
    pub errors_on_transmitted: u64,
    pub total_errors_on_transmitted: u64,
}

#[derive(Debug, Default)]
pub struct Disk {
    pub file_system: String,
    pub mount_point: PathBuf,
    pub total_space: u64,
    pub available_space: u64,
}

pub enum Request {
    Cpu { tx: oneshot::Sender<Vec<Cpu>> },
    Memory { tx: oneshot::Sender<Memory> },
    Network { tx: oneshot::Sender<Vec<Network>> },
    Disk { tx: oneshot::Sender<Vec<Disk>> },
}

pub struct SystemMonitor {
    req_tx: flume::Sender<Request>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let system = Arc::new(RwLock::new(sysinfo::System::new()));
        {
            let cpu_system = system.clone();
            thread::spawn(move || loop {
                {
                    let mut system = cpu_system.write();
                    system.refresh_cpu();
                }
                thread::sleep(sysinfo::System::MINIMUM_CPU_UPDATE_INTERVAL);
            });
        }
        {
            let mem_system = system.clone();
            thread::spawn(move || loop {
                {
                    let mut system = mem_system.write();
                    system.refresh_memory();
                }
                thread::sleep(Duration::from_secs(10));
            });
        }
        {
            let disk_system = system.clone();
            thread::spawn(move || loop {
                {
                    let mut system = disk_system.write();
                    system.refresh_disks_list();
                    system.refresh_disks();
                }
                thread::sleep(Duration::from_secs(30));
            });
        }
        {
            let net_system = system.clone();
            thread::spawn(move || loop {
                {
                    let mut system = net_system.write();
                    system.refresh_networks_list();
                    system.refresh_networks();
                }
                thread::sleep(Duration::from_millis(500));
            });
        }
        let (req_tx, req_rx) = flume::bounded(1);
        {
            let async_system = system.clone();
            thread::spawn(move || {
                while let Ok(req) = req_rx.recv() {
                    let system = async_system.read();
                    match req {
                        Request::Cpu { tx } => {
                            tx.send(
                                system
                                    .cpus()
                                    .iter()
                                    .map(|c| Cpu {
                                        name: c.name().to_string(),
                                        usage: c.cpu_usage(),
                                    })
                                    .collect(),
                            )
                            .ok();
                        }
                        Request::Memory { tx } => {
                            tx.send(Memory {
                                total: system.total_memory(),
                                available: system.available_memory(),
                                free: system.free_memory(),
                                used: system.used_memory(),
                                swap: Swap {
                                    total: system.total_swap(),
                                    free: system.free_swap(),
                                    used: system.used_swap(),
                                },
                            })
                            .ok();
                        }
                        Request::Network { tx } => {
                            tx.send(
                                system
                                    .networks()
                                    .into_iter()
                                    .map(|n| Network {
                                        name: n.0.to_string(),
                                        received: n.1.received(),
                                        total_received: n.1.total_received(),
                                        transmitted: n.1.transmitted(),
                                        total_transmitted: n.1.total_transmitted(),
                                        packets_received: n.1.packets_received(),
                                        total_packets_received: n.1.total_packets_received(),
                                        packets_transmitted: n.1.packets_transmitted(),
                                        total_packets_transmitted: n.1.total_packets_transmitted(),
                                        errors_on_received: n.1.errors_on_received(),
                                        total_errors_on_received: n.1.total_errors_on_received(),
                                        errors_on_transmitted: n.1.errors_on_transmitted(),
                                        total_errors_on_transmitted: n
                                            .1
                                            .total_errors_on_transmitted(),
                                    })
                                    .collect(),
                            )
                            .ok();
                        }
                        Request::Disk { tx } => {
                            tx.send(
                                system
                                    .disks()
                                    .iter()
                                    .map(|d| Disk {
                                        file_system: String::from_utf8_lossy(d.file_system())
                                            .to_string(),
                                        mount_point: d.mount_point().to_path_buf(),
                                        total_space: d.total_space(),
                                        available_space: d.available_space(),
                                    })
                                    .collect(),
                            )
                            .ok();
                        }
                    }
                }
            });
        }
        Self { req_tx }
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitor {
    pub async fn get_cpu(&self) -> Vec<Cpu> {
        let (tx, rx) = oneshot::channel();
        self.req_tx.send_async(Request::Cpu { tx }).await.ok();
        rx.await.unwrap_or_default()
    }

    pub async fn get_memory(&self) -> Memory {
        let (tx, rx) = oneshot::channel();
        self.req_tx.send_async(Request::Memory { tx }).await.ok();
        rx.await.unwrap_or_default()
    }

    pub async fn get_network(&self) -> Vec<Network> {
        let (tx, rx) = oneshot::channel();
        self.req_tx.send_async(Request::Network { tx }).await.ok();
        rx.await.unwrap_or_default()
    }

    pub async fn get_disk(&self) -> Vec<Disk> {
        let (tx, rx) = oneshot::channel();
        self.req_tx.send_async(Request::Disk { tx }).await.ok();
        rx.await.unwrap_or_default()
    }
}
