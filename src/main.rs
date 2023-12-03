use anyhow::{Ok, Result};
use clap::Parser;
use octocrab::Octocrab;
use polars::io::{csv::CsvWriter, SerWriter};

mod cli;
mod duration_unit;
mod workflows;

use cli::Args;
use workflows::{get_workflow_run_statistics, get_workflow_runs_and_timings};

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
    } = Args::parse();

    let client = Octocrab::builder().personal_token(github_token).build()?;

    let (all_workflow_runs, all_workflow_timings) =
        get_workflow_runs_and_timings(client, &username, &repo, &workflow_name, since).await?;

    let mut stats =
        get_workflow_run_statistics(&all_workflow_runs, &all_workflow_timings, duration_unit)?;

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
