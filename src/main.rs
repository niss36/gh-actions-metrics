use anyhow::{Ok, Result};
use clap::Parser;
use octocrab::Octocrab;
use polars::io::{csv::CsvWriter, SerWriter};
use tokio;

mod cli;
mod duration_unit;
mod workflows;

use cli::Args;

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

    let mut stats = workflows::compute_workflow_stats(
        client,
        &username,
        &repo,
        &workflow_name,
        since,
        duration_unit,
    )
    .await?;

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
