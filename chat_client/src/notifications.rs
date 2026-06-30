#![allow(unused)]

use std::{collections::VecDeque, sync::LazyLock};

use ratatui::{
    style::Stylize,
    text::{Line, Span},
};
use strum::Display;
use tokio::sync::{Mutex, broadcast};

use crate::consts::{CHANNEL_BUFFER_SIZE, NOTIFICATION_POLLER_TIMEOUT};

// In a way this is just stripped down `LogLevel`
#[derive(Debug, Clone, Display)]
pub enum NotificationType {
    Info,
    Warn,
    Error,
}

pub type Notification = (NotificationType, String);

/// The channel where the notifications will be sent,
/// in addition to the notification buffer
pub static NOTIFICATION_CHANNEL: LazyLock<broadcast::Sender<Notification>> =
    LazyLock::new(|| broadcast::channel(CHANNEL_BUFFER_SIZE).0);

pub fn subscribe() -> broadcast::Receiver<Notification> {
    NOTIFICATION_CHANNEL.subscribe()
}

/// This function panics if called outside of the tokio runtime
#[allow(clippy::needless_pass_by_value)]
fn add_notification(typ: NotificationType, data: impl ToString) {
    let data = data.to_string();
    tokio::spawn(async {
        let notif = (typ, data);
        NOTIFICATION_CHANNEL.send(notif);
    });
}

#[macro_export]
macro_rules! notif_info {
    ($fmt:expr $(, $args:expr)*) => {
        let msg = format!($fmt, $($args),*);
        $crate::notifications::info(msg);
    };
}

#[macro_export]
macro_rules! notif_warn {
    ($fmt:expr $(, $args:expr)*) => {
        let msg = format!($fmt, $($args),*);
        $crate::notifications::warn(msg);
    };
}

#[macro_export]
macro_rules! notif_error {
    ($fmt:expr $(, $args:expr)*) => {
        let msg = format!($fmt, $($args),*);
        $crate::notifications::error(msg);
    };
}

pub fn info(data: impl ToString) {
    add_notification(NotificationType::Info, data);
}

pub fn warn(data: impl ToString) {
    add_notification(NotificationType::Warn, data);
}

pub fn error(data: impl ToString) {
    add_notification(NotificationType::Error, data);
}

pub fn notification_to_span(notif: &Notification) -> Span<'_> {
    let (typ, msg) = notif;
    let out = format!("{typ}: {msg}");
    let out = Span::from(out);
    match notif.0 {
        NotificationType::Info => out.green(),
        NotificationType::Warn => out.yellow(),
        NotificationType::Error => out.red(),
    }
}
