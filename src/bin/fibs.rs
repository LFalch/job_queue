use std::{env::args, io::stdin, num::NonZeroUsize};

use job_queue::{ThreadPool, JobHandler};

fn fib(n: u64) -> u64 {
  if n < 2 {
      1
    } else {
        fib(n-1) + fib(n-2)
    }
}

#[derive(Debug, Clone, Copy)]
struct FibJob;

impl JobHandler<u64> for FibJob {
    fn handle(&mut self, n: u64) {
        let fibs = fib(n);
        println!("fib({}) = {}", n, fibs);
    }
}

fn main() {
    let mut args = args().skip(1);
    let n = args
        .next()
        .and_then(|n| n.parse::<NonZeroUsize>().ok());
    let cap = if n.is_none() { None } else {
        args.next().and_then(|n| n.parse::<NonZeroUsize>().ok())
    };

    let n = if let Some(n) = n {
        n.get()
    } else {
        eprintln!("Please provide a valid thread count");

        return
    };

    let pool = if let Some(cap) = cap {
        ThreadPool::with_capacity(cap, n, FibJob)
    } else {
        ThreadPool::new(n, FibJob)
    };

    let mut line = String::new();

    loop {
        let n = stdin().read_line(&mut line).unwrap();
        if n == 0 { break }

        if let Ok(n) = line.trim().parse() {
            pool.add_job(n);
        }
        line.clear();
    }
    eprintln!("Waiting for jobs to finish.");
}
