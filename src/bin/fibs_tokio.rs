use std::{future::Future, pin::Pin};
use tokio::{try_join, spawn};

fn fib(n: u64) -> Pin<Box<dyn 'static + Send + Future<Output = u64>>> {
  if n < 2 {
        Box::pin(async { 1 })
    } else {
        Box::pin(async move {
            let (a, b) = try_join!(
                spawn(fib(n-1)),
                spawn(fib(n-2))
            ).unwrap();
            a + b
        })
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    let n = 30;

    let m = fib(n).await;

    println!("fib({}) = {}", n, m);
}
