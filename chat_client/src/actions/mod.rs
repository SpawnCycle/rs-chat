#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The actions are not run in a tui, so they need to be able to output stuff to stdout and stderr"
)]

mod ls;

use crate::{
    actions::ls::ls_action,
    config::{AppAction, AppConfig},
};

/// executes the action
///
/// # Errors
///
/// this function errors if the underlying action implementation erorrs
pub async fn actions(config: AppConfig, action: AppAction) -> anyhow::Result<()> {
    match action {
        AppAction::Ls(args) => ls_action(config, args).await?,
    }
    Ok(())
}
