use anyhow::{anyhow, Ok, Result};
use std::{sync::mpsc, thread, time::Duration};

const NUM_PRODUCERS: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}

impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // 创建 producers
    for idx in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(idx, tx));
    }

    drop(tx); // 手动关闭 tx，否则 rx 会一直等待

    // 创建 consumer
    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consumer: {:?}", msg);
        }
        println!("consumer exit");
        42
    });

    let secret = consumer
        .join()
        .map_err(|e| anyhow!("Thread join error: {:?}", e))?;

    println!("secret: {}", secret);
    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>();
        tx.send(Msg::new(idx, value))?;
        let sleep_time = rand::random::<u64>() % 1000;
        thread::sleep(Duration::from_millis(sleep_time));

        if rand::random::<u8>() % 10 == 0 {
            break;
        }
    }
    println!("thread {} exit", idx);
    Ok(())
}
