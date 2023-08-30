use std::time::Duration;

use system_monitor::SystemMonitorBuilder;
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let monitor = SystemMonitorBuilder::new()
        .cpu(Duration::from_secs(1))
        .memory(Duration::from_secs(10))
        .network(Duration::from_secs(1))
        .disk(Duration::from_secs(30))
        .build();
    time::sleep(Duration::from_secs(60)).await;
    println!("{:?}", monitor.get_cpu().await);
    println!("{:?}", monitor.get_memory().await);
    println!("{:?}", monitor.get_network().await);
    println!("{:?}", monitor.get_disk().await);
    Ok(())
}
