use anyhow::{Ok, Result};
use chrono::NaiveDate;
use octocrab::{models::workflows::Run, Octocrab, Page};
use polars::{
    df,
    lazy::dsl::{col, count, lit},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{duration_unit::DurationUnit, granularity::Granularity};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowRunTiming {
    run_duration_ms: u64,
}

pub async fn get_workflow_runs_and_timings(
    client: Octocrab,
    owner: &str,
    repo: &str,
    workflow_name: &str,
    since: Option<NaiveDate>,
) -> Result<(Vec<Run>, Vec<WorkflowRunTiming>)> {
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

    Ok((all_workflow_runs, all_workflow_timings))
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

pub fn compute_workflow_statistics(
    runs: &[Run],
    timings: &[WorkflowRunTiming],
    duration_unit: DurationUnit,
    granularity: Granularity,
) -> Result<DataFrame> {
    let statuses: Vec<_> = runs
        .iter()
        .map(|run| run.conclusion.clone().unwrap_or_default())
        .collect();
    let dates: Vec<_> = runs
        .iter()
        .map(|run| run.created_at.clone().format("%Y-%m-%d").to_string())
        .collect();
    let durations: Vec<_> = timings
        .iter()
        .map(|timing| duration_unit.convert(timing.run_duration_ms))
        .collect();

    let df = df![
        "status" => statuses,
        "date" => dates,
        "duration" => durations
    ]?;

    fn compute_stats(groups: LazyGroupBy) -> LazyFrame {
        groups.agg([
            count().alias("count"),
            col("duration").mean().round(2).alias("mean_duration"),
            col("duration").median().round(2).alias("median_duration"),
            col("duration")
                .quantile(lit(0.95), QuantileInterpolOptions::Nearest)
                .round(2)
                .alias("p95_duration"),
        ])
    }

    let stats = match granularity {
        Granularity::Daily => {
            let groups = df.lazy().group_by([col("date"), col("status")]);

            compute_stats(groups)
                .sort("date", Default::default())
                .collect()?
        }
        Granularity::Total => {
            let groups = df.lazy().group_by([col("status")]);

            compute_stats(groups).collect()?
        }
    };

    Ok(stats)
}
