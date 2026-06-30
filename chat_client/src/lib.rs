pub mod app;
pub mod components;
pub mod config;
pub mod consts;
pub mod event;

mod actions;
mod chat;
mod helper;
mod logs;
mod notifications;
mod requests;
mod room;
mod task;
mod ws_handler;

use std::io;

use crossterm::event::Event;
use thiserror::Error;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::consts::{POLL_DURATION, TICK_DURATION};

pub use crate::actions::actions;

#[derive(Debug)]
pub enum AppEvent {
    Tick,
    Event(Event),
    /// An error in one of the tasks that's serious enough to be propogated up
    Error(AppError),
}

#[derive(Debug, Error)]
pub enum AppError {
    /// An error in the event poller.
    /// This is a failure state
    #[error("{0}")]
    Event(#[from] io::Error),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[must_use]
pub fn start_event_poller(tx: Sender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let Ok(res) =
                tokio::task::spawn_blocking(|| crossterm::event::poll(POLL_DURATION)).await
            else {
                break;
            };
            let has_elements = match res {
                Ok(val) => val,
                Err(err) => {
                    let _ = tx.send(AppEvent::Error(AppError::Event(err))).await;
                    break;
                }
            };
            if has_elements {
                match crossterm::event::read() {
                    Ok(ev) => {
                        let _ = tx.send(AppEvent::Event(ev)).await;
                    }
                    Err(err) => {
                        let _ = tx.send(AppEvent::Error(AppError::Event(err))).await;
                        break;
                    }
                }
            }
        }
    })
}

#[must_use]
pub fn start_tick_poller(tx: Sender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(TICK_DURATION).await;
            let _ = tx.send(AppEvent::Tick).await;
        }
    })
}
