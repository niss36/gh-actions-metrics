use anyhow::{Ok, Result};
use clap::Parser;
use octocrab::Octocrab;
use polars::io::{csv::CsvWriter, SerWriter};

mod cli;
mod duration_unit;
mod granularity;
mod workflows;

use cli::Args;
use workflows::{compute_workflow_statistics, get_workflow_runs_and_timings};

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        github_token,
        username,
        repo,
        workflow_name,
        csv,
        since,
        duration_unit,
        granularity,
    } = Args::parse();

    let client = Octocrab::builder().personal_token(github_token).build()?;

    let (runs, timings) =
        get_workflow_runs_and_timings(client, &username, &repo, &workflow_name, since).await?;

    let mut stats = compute_workflow_statistics(&runs, &timings, duration_unit, granularity)?;

    if csv {
        CsvWriter::new(std::io::stdout())
            .include_header(true)
            .with_separator(b',')
            .finish(&mut stats)?;
    } else {
        println!("{stats}");
    }

    Ok(())
}
