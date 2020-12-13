use std::future::Future;
use std::thread;
use std::time::Instant;
use std::mem;

use criterion::{black_box, criterion_main, BatchSize, Criterion};
use event_listener::Event;
use futures_lite::future::{block_on, FutureExt as _, yield_now};

macro_rules! benchmark_channel {
    (
        $criterion:ident,
        $name:literal,
        $create:expr,
        $send:expr,
        $recv:expr,
    ) => {
        for &bound in &[
            Some(1),
            Some(2),
            Some(4),
            Some(8),
            Some(16),
            Some(32),
            Some(64),
            Some(128),
            None,
        ] {
            let bound_str = match bound {
                Some(bound) => format!("bounded({})", bound),
                None => "unbounded".to_owned(),
            };
            let bench_name = |desc: &str| format!("{} {} {}", desc, $name, bound_str);

            $criterion.bench_function(&bench_name("oneshot send"), |b| {
                b.iter_batched(
                    || $create(bound),
                    |(tx, rx)| {
                        black_box(block_on($send(&tx, 5)).unwrap());
                        (tx, rx)
                    },
                    BatchSize::SmallInput,
                );
            });
            $criterion.bench_function(&bench_name("oneshot recv"), |b| {
                b.iter_batched(
                    || {
                        let (tx, rx) = $create(bound);
                        block_on($send(&tx, 5)).unwrap();
                        (tx, rx)
                    },
                    |(tx, rx)| {
                        black_box(block_on($recv(&rx)).unwrap());
                        (tx, rx)
                    },
                    BatchSize::SmallInput,
                );
            });

            for &threads in &[0, 1, 2, 4] {
                let contended_str = match threads {
                    0 => "uncontended".to_owned(),
                    threads => format!("contended({})", threads),
                };
                let bench_name = |desc| bench_name(&format!("{} {}", desc, contended_str));

                $criterion.bench_function(&bench_name("mpmc send"), |b| {
                    let (tx, rx) = $create(bound);

                    let mut tasks = Tasks::new();

                    // Have one thread draining the channel so it doesn't get clogged up
                    tasks.spawn(async move {
                        while $recv(&rx).await.is_ok() {
                            yield_now().await;
                        }
                    });
                    // Competing threads
                    for _ in 0..threads {
                        let tx = tx.clone();
                        tasks.spawn(async move {
                            while $send(&tx, 5).await.is_ok() {
                                yield_now().await;
                            }
                        });
                    }

                    b.iter(|| {
                        block_on($send(&tx, 5)).unwrap();
                    });
                });
                $criterion.bench_function(&bench_name("mpmc recv"), |b| {
                    let (tx, rx) = $create(bound);

                    let mut tasks = Tasks::new();

                    // Have one thread filling the channel with values
                    tasks.spawn(async move {
                        while $send(&tx, 5).await.is_ok() {
                            yield_now().await;
                        }
                    });
                    // Competing threads
                    for _ in 0..threads {
                        let rx = rx.clone();
                        tasks.spawn(async move {
                            while $recv(&rx).await.is_ok() {
                                yield_now().await;
                            }
                        });
                    }

                    b.iter(|| {
                        black_box(block_on($recv(&rx)).unwrap());
                    });
                });
            }
        }
    };
}

fn benchmark() {
    let started = Instant::now();

    let mut c = Criterion::default().configure_from_args();

    benchmark_channel! {
        c,
        "flume",
        |bound: Option<usize>| bound.map_or_else(flume::unbounded::<u32>, flume::bounded::<u32>),
        flume::Sender::send_async,
        flume::Receiver::recv_async,
    }
    benchmark_channel! {
        c,
        "async-channel",
        |bound: Option<usize>| bound.map_or_else(async_channel::unbounded::<u32>, async_channel::bounded::<u32>),
        async_channel::Sender::send,
        async_channel::Receiver::recv,
    }

    let time = started.elapsed();
    println!("Completed in {} seconds", time.as_secs());
}

criterion_main!(benchmark);

struct Tasks {
    stop: Event,
    threads: Vec<thread::JoinHandle<()>>,
}
impl Tasks {
    fn new() -> Self {
        Self {
            stop: Event::new(),
            threads: Vec::new(),
        }
    }
    fn spawn(&mut self, f: impl Future<Output = ()> + Send + 'static) {
        let stop = self.stop.listen();
        self.threads.push(thread::spawn(move || block_on(f.or(stop))));
    }
}
impl Drop for Tasks {
    fn drop(&mut self) {
        self.stop.notify(usize::MAX);
        for thread in mem::take(&mut self.threads) {
            thread.join().unwrap();
        }
    }
}
