use anyhow::{Ok, Result};
use chrono::NaiveDate;
use octocrab::{models::workflows::Run, Octocrab, Page};
use polars::df;
use polars::lazy::dsl::col;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

use crate::duration_unit::DurationUnit;

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowRunTiming {
    run_duration_ms: u64,
}

pub async fn compute_workflow_stats(
    client: Octocrab,
    owner: &str,
    repo: &str,
    workflow_name: &str,
    since: Option<NaiveDate>,
    duration_unit: DurationUnit,
) -> Result<DataFrame> {
    let all_workflow_runs =
        get_all_workflow_runs(&client, owner, repo, workflow_name, since).await?;

    let mut all_workflow_timings = Vec::with_capacity(all_workflow_runs.len());

    for workflow_run in &all_workflow_runs {
        let run_id = workflow_run.id.0;

        let response: WorkflowRunTiming = client
            .get(
                format!("/repos/{owner}/{repo}/actions/runs/{run_id}/timing"),
                None::<&()>,
            )
            .await?;

        all_workflow_timings.push(response);
    }

    get_workflow_run_statistics(&all_workflow_runs, &all_workflow_timings, duration_unit)
}

async fn get_all_workflow_runs(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    workflow_name: &str,
    since: Option<NaiveDate>,
) -> Result<Vec<Run>> {
    let workflow_runs_page = client
        .workflows(owner, repo)
        .list_runs(workflow_name)
        .status("completed")
        .send()
        .await?;

    match since {
        None => Ok(client.all_pages(workflow_runs_page).await?),
        Some(date) => Ok(all_pages_since(client, workflow_runs_page, date)
            .await?
            .into_iter()
            .filter(|run| run.created_at.date_naive() >= date)
            .collect()),
    }
}

async fn all_pages_since(
    client: &Octocrab,
    mut page: Page<Run>,
    date: NaiveDate,
) -> Result<Vec<Run>> {
    let mut runs = page.take_items();
    if let Some(last) = runs.last() {
        if last.created_at.date_naive() < date {
            return Ok(runs);
        }
    }

    while let Some(mut next_page) = client.get_page(&page.next).await? {
        runs.append(&mut next_page.take_items());
        if let Some(last) = runs.last() {
            if last.created_at.date_naive() < date {
                return Ok(runs);
            }
        }

        page = next_page;
    }

    Ok(runs)
}

fn get_workflow_run_statistics(
    workflow_runs: &[octocrab::models::workflows::Run],
    workflow_timings: &[WorkflowRunTiming],
    duration_unit: DurationUnit,
) -> Result<DataFrame> {
    let workflow_statuses: Vec<_> = workflow_runs
        .iter()
        .map(|run| run.conclusion.clone().unwrap_or_default())
        .collect();
    let workflow_dates: Vec<_> = workflow_runs
        .iter()
        .map(|run| run.created_at.clone().format("%Y-%m-%d").to_string())
        .collect();
    let workflow_durations: Vec<_> = workflow_timings
        .iter()
        .map(|timing| duration_unit.convert(timing.run_duration_ms))
        .collect();

    let df = df![
        "status" => workflow_statuses,
        "date" => workflow_dates,
        "duration" => workflow_durations
    ]?;

    let stats = df
        .lazy()
        .group_by([col("date"), col("status")])
        .agg([
            col("*").count().alias("count"),
            col("duration").mean().round(2).alias("mean_duration"),
            col("duration").median().round(2).alias("median_duration"),
            col("duration")
                .quantile(lit(0.95), QuantileInterpolOptions::Nearest)
                .round(2)
                .alias("p95_duration"),
        ])
        .sort("date", Default::default())
        .collect()?;

    Ok(stats)
}
