mod client;

use anyhow::Result;
use client::TableRaiserClient;
use std::{io::Write, time::Duration};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = TableRaiserClient::new().await;

    tokio::select! {
        _ = client.wait_closed() => {
            println!("Client is closed, exiting...");
        }
        result = run(&client) => {
            result?;
            println!("App is done")
        }
    };

    Ok(())
}

async fn run(client: &TableRaiserClient) -> Result<()> {
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        match input.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["ping", d] if let Ok(id) = d.parse() => {
                let response = client.ping(id).await?;
                println!("pong {response}");
            }
            ["wake", d] if let Ok(ms) = d.parse() => {
                client.wake(Duration::from_millis(ms)).await?;
            }
            ["exit"] => {
                return Ok(());
            }
            ["help"] => print_help(),
            _ => {
                println!("Unknown command: \"{}\"", input.trim());
                print_help();
            }
        }
    }

    fn print_help() {
        println!("Available commands: ");
        println!("ping <id>: ping with <id> expecting the same as response");
        println!("wake <d>: pull wake pin high for <d> ms");
        println!("exit: disconnects and exits the cli");
    }
}
