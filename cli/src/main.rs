mod client;

use anyhow::Result;
use client::TableRaiserClient;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = TableRaiserClient::new().await;

    tokio::select! {
        _ = client.wait_closed() => {
            println!("Client is closed, exiting...");
        }
        result = run(&client) => {
            _ = result?;
            println!("App is done")
        }
    };

    Ok(())
}

async fn run(client: &TableRaiserClient) -> Result<()> {
    let mut ticker = interval(Duration::from_millis(250));

    for i in 0..10 {
        ticker.tick().await;
        print!("Pinging with {i}... ");
        let res = client.ping(i).await?;
        println!("got {res}!");
        assert_eq!(res, i);
    }

    Ok(())
}
