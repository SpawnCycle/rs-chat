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

pub static PENDING_NOTIFICATIONS: LazyLock<Mutex<VecDeque<Notification>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub static NOTIFICATIONS: LazyLock<Mutex<Vec<Notification>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// This function will try to get the lock for the notifications,
/// and will return `None` if it's not free
#[allow(unused)]
pub fn try_get_notifications() -> Option<Vec<Notification>> {
    let notifs = NOTIFICATIONS.try_lock().ok()?;

    Some(notifs.clone())
}

pub fn subscribe() -> broadcast::Receiver<Notification> {
    NOTIFICATION_CHANNEL.subscribe()
}

/// This function will block the current thread until it can get
/// the lock for the notifications
///
/// # Panics
///
/// This function panics if called in an async context
///
/// Use `spawn_blocking` and the likes to use this function in an async context
pub fn get_notifications() -> Vec<Notification> {
    let notifs = NOTIFICATIONS.blocking_lock();

    notifs.clone()
}

/// This function panics if called outside of the tokio runtime
#[allow(clippy::needless_pass_by_value)]
fn add_notification(typ: NotificationType, data: impl ToString) {
    let data = data.to_string();
    tokio::spawn(async {
        let mut notifs = PENDING_NOTIFICATIONS.lock().await;
        let notif = (typ, data);
        notifs.push_front(notif.clone());
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

pub fn start_notification_poller() {
    tokio::spawn(async {
        loop {
            get_pending().await;
            tokio::time::sleep(NOTIFICATION_POLLER_TIMEOUT).await;
        }
    });
}

pub async fn get_pending() {
    let mut notifications = NOTIFICATIONS.lock().await;
    let mut pending = PENDING_NOTIFICATIONS.lock().await;
    while let Some(notif) = pending.pop_back() {
        notifications.push(notif);
    }
}

pub fn notification_to_line(notif: &Notification) -> Line<'_> {
    let (typ, msg) = notif;
    let out = format!("{typ}: {msg}");
    let out = Span::from(out);
    let format = match notif.0 {
        NotificationType::Info => out.green(),
        NotificationType::Warn => out.yellow(),
        NotificationType::Error => out.red(),
    };

    format.into()
}
