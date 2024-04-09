use std::{pin::Pin, time::Duration};

use futures::Future;
use tokio::time::sleep;

pub async fn async_retry<'a, F, R, E>(f: F, max_retries: i32) -> Result<R, E>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>,
    E: std::fmt::Debug + std::fmt::Display,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(r) => {
                return Ok(r);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                retries += 1;

                if retries > max_retries {
                    return Err(e);
                }

                eprintln!("Error, retrying in 2 seconds");
                sleep(Duration::from_secs(2)).await;

                continue;
            }
        }
    }
}
