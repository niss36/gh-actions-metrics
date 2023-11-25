use chrono::NaiveDate;
use clap::Parser;

use crate::duration_unit::DurationUnit;

#[derive(Parser, Debug)]
pub(crate) struct Args {
    #[clap(long, env, hide_env_values(true), verbatim_doc_comment)]
    /// The GitHub Token required to access the API
    ///
    /// Can be one of the following:
    /// - A Personal Access Token
    /// - A User Access Token
    /// - The token provided within GitHub Actions
    pub(crate) github_token: String,

    pub(crate) username: String,

    pub(crate) repo: String,

    pub(crate) workflow_name: String,

    #[clap(long)]
    /// Write the statistics in CSV format to standard output
    pub(crate) csv: bool,

    #[clap(long)]
    /// Filter workflow runs since this date in ISO format, e.g. 2023-01-01
    pub(crate) since: Option<NaiveDate>,

    #[clap(long, default_value_t)]
    pub(crate) duration_unit: DurationUnit,
}
