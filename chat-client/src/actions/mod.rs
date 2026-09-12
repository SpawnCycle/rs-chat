#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The actions are not run in a tui, so they need to be able to output stuff to stdout and stderr"
)]

mod echo;
mod ls;

use crate::{
    actions::{echo::echo_action, ls::ls_action},
    config::{ActionType, AppConfig},
};

/// executes the action
///
/// # Errors
///
/// this function errors if the underlying action implementation erorrs
pub async fn actions(config: AppConfig, action: ActionType) -> anyhow::Result<()> {
    match action {
        ActionType::Ls(args) => ls_action(config, args).await?,
        ActionType::Echo(args) => echo_action(config, args).await?,
    }
    Ok(())
}
