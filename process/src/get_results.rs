use std::path::Path;
use std::io;

use anyhow::{bail, Context as _, anyhow};

use super::{Channel, Channels, Contentions, Mpmc, Oneshot, Results};

const CRITERION_PATH: &str = if cfg!(windows) {
    "..\\target\\criterion"
} else {
    "../target/criterion"
};

pub(super) fn get_results() -> anyhow::Result<Results> {
    Ok(Results {
        oneshot: Oneshot {
            send: get_channels("oneshot send")?,
            recv: get_channels("oneshot recv")?,
        },
        mpmc: Mpmc {
            send: get_contentions("mpmc send")?,
            recv: get_contentions("mpmc recv")?,
        },
    })
}

fn get_contentions(prefix: &str) -> anyhow::Result<Contentions> {
    Ok(Contentions(
        [0, 1, 2, 4]
            .iter()
            .map(|&contention| {
                Ok((
                    contention,
                    get_channels(&match contention {
                        0 => format!("{} uncontended", prefix),
                        contention => format!("{} contended({})", prefix, contention),
                    })?,
                ))
            })
            .collect::<anyhow::Result<_>>()?,
    ))
}

fn get_channels(prefix: &str) -> anyhow::Result<Channels> {
    Ok(Channels {
        async_channel: get_channel(&format!("{} async-channel", prefix))?,
        flume: get_channel(&format!("{} flume", prefix))?,
    })
}

fn get_channel(prefix: &str) -> anyhow::Result<Channel> {
    Ok(Channel {
        bounded: [1, 2, 4, 8, 16, 32, 64, 128]
            .iter()
            .map(|&bound| Ok((bound, get_value(&format!("{} bounded({})", prefix, bound))?)))
            .collect::<anyhow::Result<_>>()?,
        unbounded: get_value(&format!("{} unbounded", prefix))?,
    })
}

fn get_value(bench_name: &str) -> anyhow::Result<f64> {
    let mut path = Path::new(CRITERION_PATH).to_owned();
    path.push(bench_name);
    path.push("new");
    path.push("raw.csv");

    let mut total = 0.;
    let mut count = 0.;

    for (i, record) in csv::Reader::from_path(&path)
        .map_err(|e| {
            match e.kind() {
                csv::ErrorKind::Io(e) if e.kind() == io::ErrorKind::NotFound => {
                    anyhow!("Benchmarks have not been run")
                },
                _ => anyhow::Error::from(e),
            }
            .context(format!("Failed to open '{}'", path.display()))
        })?
        .records()
        .enumerate()
    {
        total += record
            .map_err(anyhow::Error::from)
            .and_then(read_csv_record)
            .with_context(|| format!("In '{}' line {}", path.display(), i + 2))?;
        count += 1.;
    }

    Ok(total / count)
}

fn read_csv_record(record: csv::StringRecord) -> anyhow::Result<f64> {
    if record.len() < 8 {
        bail!("Expected 8 fields in record, found {}", record.len());
    }

    let raw_sampled: f64 = record[5].parse().context("Sample measured value")?;
    let sampled = raw_sampled
        / match &record[6] {
            "ns" => 1.0,
            "us" => 1_000.0,
            "ms" => 1_000_000.0,
            "s" => 1_000_000_000.0,
            _ => bail!("Unknown unit {}", &record[6]),
        };
    let iteration_count: f64 = record[7].parse().context("Iteration count")?;

    Ok(sampled / iteration_count)
}
