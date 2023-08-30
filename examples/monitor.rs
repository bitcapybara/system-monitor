use std::time::Duration;

use system_monitor::SystemMonitor;
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let monitor = SystemMonitor::new();
    time::sleep(Duration::from_secs(60)).await;
    println!("{:?}", monitor.get_cpu().await);
    println!("{:?}", monitor.get_memory().await);
    println!("{:?}", monitor.get_network().await);
    println!("{:?}", monitor.get_disk().await);
    Ok(())
}
