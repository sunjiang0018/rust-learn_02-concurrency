use std::{thread, time::Duration};

use anyhow::{Ok, Result};
use concurrency::CmapMetrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    let metrics = CmapMetrics::new();

    println!("{}", metrics);

    // start N workers and M requesters

    for idx in 0..N {
        task_worker(idx, metrics.clone())?;
    }

    for _ in 0..M {
        request_worker(metrics.clone())?;
    }

    loop {
        thread::sleep(Duration::from_secs(1));
        println!("{}", metrics);
    }
}

fn task_worker(idx: usize, metrics: CmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
            metrics.inc(format!("call.thread.wroker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        anyhow::Ok(())
    });
    Ok(())
}

fn request_worker(metrics: CmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();

            let page = rng.gen_range(1..5);
            thread::sleep(Duration::from_millis(rng.gen_range(50..800)));
            metrics.inc(format!("req.pages.{}", page))?;
        }
        #[allow(unreachable_code)]
        Ok(())
    });
    Ok(())
}
