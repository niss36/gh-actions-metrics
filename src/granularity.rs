use std::fmt::Display;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Granularity {
    Daily,
    Total,
}

impl Display for Granularity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Granularity::Daily => "daily",
            Granularity::Total => "total",
        })
    }
}
