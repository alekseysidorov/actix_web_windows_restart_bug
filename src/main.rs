use actix_bug::ApiManager;
use actix_rt::time::{delay_for, timeout};
use futures::{prelude::*, stream};

use std::{
    io,
    net::TcpListener,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

async fn run_manager() -> io::Result<()> {
    let mut manager = ApiManager::new();

    let active_time = 15;
    loop {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        manager.start_server(listener).await?;
        log::info!("Server will active for the next {} seconds.", active_time);
        delay_for(Duration::from_secs(active_time)).await;
        manager.stop_server().await;

        log::info!("Server restart requested");
    }
}

async fn make_ping() -> anyhow::Result<String> {
    let text = reqwest::get("http://127.0.0.1:8080/ping")
        .await?
        .text()
        .await?;

    if text != "pong" {
        anyhow::bail!("Wrong ping response: {}", text)
    } else {
        Ok(text)
    }
}

async fn run_client() {
    let successes = AtomicU32::new(0);
    let fails = AtomicU32::new(0);

    let workers = 50;

    stream::repeat(())
        .for_each_concurrent(Some(workers), |_| async {
            delay_for(Duration::from_millis(100)).await;
            let result = timeout(Duration::from_secs(5), make_ping()).await;

            match result {
                Ok(result) => {
                    let value = successes.fetch_add(1, Ordering::SeqCst);
                    if value % workers as u32 == 0 {
                        log::info!(
                            "Performed {} ping requests, response: {:?}",
                            value + 1,
                            result
                        );
                    }
                }

                Err(err) => {
                    let value = fails.fetch_add(1, Ordering::SeqCst);
                    if value % workers as u32 == 0 {
                        log::error!("{} requests has been timeouted: {}", value + 1, err);
                    }
                }
            }
        })
        .await;
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "warn,actix_bug=trace");
    env_logger::init();

    actix_rt::spawn(run_client());
    run_manager().await.map_err(From::from)
}
