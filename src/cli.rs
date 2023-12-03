use chrono::NaiveDate;
use clap::Parser;

use crate::{duration_unit::DurationUnit, granularity::Granularity};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, env, hide_env_values(true), verbatim_doc_comment)]
    /// The GitHub Token required to access the API
    ///
    /// Can be one of the following:
    /// - A Personal Access Token
    /// - A User Access Token
    /// - The token provided within GitHub Actions
    pub github_token: String,

    pub username: String,

    pub repo: String,

    pub workflow_name: String,

    #[clap(long)]
    /// Write the statistics in CSV format to standard output
    pub csv: bool,

    #[clap(long)]
    /// Filter workflow runs since this date in ISO format, e.g. 2023-01-01
    pub since: Option<NaiveDate>,

    #[clap(long, default_value_t)]
    pub duration_unit: DurationUnit,

    #[clap(long, default_value_t = Granularity::Daily)]
    pub granularity: Granularity,
}
