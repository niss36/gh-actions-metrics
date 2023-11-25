use std::{fmt::Display, time::Duration};

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub(crate) enum DurationUnit {
    #[default]
    Seconds,
    Minutes,
    Hours,
}

impl DurationUnit {
    pub fn convert(self, ms: u64) -> f64 {
        let secs = Duration::from_millis(ms).as_secs_f64();

        match self {
            DurationUnit::Seconds => secs,
            DurationUnit::Minutes => secs / 60.,
            DurationUnit::Hours => secs / (60. * 60.),
        }
    }
}

impl Display for DurationUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DurationUnit::Seconds => "seconds",
            DurationUnit::Minutes => "minutes",
            DurationUnit::Hours => "hours",
        })
    }
}
