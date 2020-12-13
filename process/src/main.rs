use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use anyhow::Context as _;

mod get_results;

struct Results {
    oneshot: Oneshot,
    mpmc: Mpmc,
}

struct Oneshot {
    send: Channels,
    recv: Channels,
}

struct Mpmc {
    send: Contentions,
    recv: Contentions,
}

struct Contentions(BTreeMap<usize, Channels>);

impl Contentions {
    fn write_csv(&self, to: &mut impl Write) -> io::Result<()> {
        write!(to, "contention")?;
        for channel in &Channels::NAMES {
            write!(to, ",{}", channel)?;
        }
        writeln!(to)?;

        for (contention, channels) in &self.0 {
            write!(to, "{}", contention)?;
            for value in channels.values() {
                write!(to, ",{}", value)?;
            }
            writeln!(to)?;
        }

        Ok(())
    }
}

struct Channels {
    async_channel: Channel,
    flume: Channel,
}

impl Channels {
    fn write_csv(&self, to: &mut impl Write) -> io::Result<()> {
        writeln!(to, "channel,time")?;
        for (channel, value) in Self::NAMES.iter().zip(self.values()) {
            writeln!(to, "{},{}", channel, value)?;
        }
        Ok(())
    }

    const NAMES: [&'static str; 18] = [
        "async-channel bounded(1)",
        "async-channel bounded(2)",
        "async-channel bounded(4)",
        "async-channel bounded(8)",
        "async-channel bounded(16)",
        "async-channel bounded(32)",
        "async-channel bounded(64)",
        "async-channel bounded(128)",
        "async-channel unbounded",
        "flume bounded(1)",
        "flume bounded(2)",
        "flume bounded(4)",
        "flume bounded(8)",
        "flume bounded(16)",
        "flume bounded(32)",
        "flume bounded(64)",
        "flume bounded(128)",
        "flume unbounded",
    ];

    fn values(&self) -> impl Iterator<Item = f64> + '_ {
        Iterator::chain(self.async_channel.values(), self.flume.values())
    }
}

struct Channel {
    // Must contain only 1, 2, 4, 8, 16, 32, 64, 128
    bounded: BTreeMap<usize, f64>,
    unbounded: f64,
}

impl Channel {
    fn values(&self) -> impl Iterator<Item = f64> + '_ {
        Iterator::chain(
            self.bounded.values().copied(),
            std::iter::once(self.unbounded),
        )
    }
}

fn main() -> anyhow::Result<()> {
    let results = get_results::get_results().context("Failed to get results")?;

    let data_dir = Path::new("..").join("data");
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create {}", data_dir.display()))?;

    File::create(data_dir.join("oneshot-send.csv"))
        .and_then(|file| results.oneshot.send.write_csv(&mut BufWriter::new(file)))
        .context("Failed to write oneshot send results")?;

    eprintln!("Written oneshot-send.csv");

    File::create(data_dir.join("oneshot-recv.csv"))
        .and_then(|file| results.oneshot.recv.write_csv(&mut BufWriter::new(file)))
        .context("Failed to write oneshot recv results")?;

    eprintln!("Written oneshot-recv.csv");

    File::create(data_dir.join("mpmc-send.csv"))
        .and_then(|file| results.mpmc.send.write_csv(&mut BufWriter::new(file)))
        .context("Failed to write mpmc send results")?;

    eprintln!("Written mpmc-send.csv");

    File::create(data_dir.join("mpmc-recv.csv"))
        .and_then(|file| results.mpmc.recv.write_csv(&mut BufWriter::new(file)))
        .context("Failed to write mpmc recv results")?;

    eprintln!("Written mpmc-recv.csv");

    Ok(())
}
