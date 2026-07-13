use std::num::NonZero;

use ratatui::{
    Frame,
    layout::{Constraint, Direction::Horizontal, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders},
};

use crate::{
    components::AppContext,
    event::{EventType, RoomEvent, UserLocator},
};

#[derive(Clone, Copy, Debug)]
pub enum Offset {
    /// offset from the top
    Absolute(u32),
    /// offset relative to the most recent chat
    Relative(NonZero<u32>),
}

#[must_use]
pub fn top_block<'a>() -> Block<'a> {
    Block::new().borders(Borders::BOTTOM)
}

pub fn draw_top_bar(f: &'_ mut Frame, area: Rect, name: &str, ctx: &AppContext) {
    let block = Block::new().borders(Borders::BOTTOM);
    f.render_widget(&block, area);
    let area = block.inner(area);

    let name_len = name.chars().count();
    let name = name.to_string();

    let chunks = Layout::new(
        Horizontal,
        [Constraint::Length(name_len as u16), Constraint::Fill(1)],
    )
    .split(area);

    let msg = "Press Ctrl+h to display help menu";

    let notif_count = ctx.notifications().len();
    let left = Line::from(name);
    let right = Line::from_iter([
        Span::from(notif_count.to_string()).red(),
        Span::from(" "),
        Span::from(msg),
    ])
    .right_aligned();

    f.render_widget(left, chunks[0]);
    f.render_widget(right, chunks[1]);
}

pub fn draw_room_events(
    f: &'_ mut Frame,
    area: Rect,
    chats: &[RoomEvent],
    users: &impl UserLocator,
    offset: Option<Offset>,
) {
    let chats = chats.iter();
    let height = area.height as usize;
    match offset {
        None => {
            let chats = chats.rev().take(height).rev().collect::<Vec<_>>();
            draw_lines(f, area, &chats, users, true);
        }
        Some(Offset::Relative(offset)) => {
            let chats = chats
                .rev()
                .skip(offset.get() as usize)
                .take(height)
                .rev()
                .collect::<Vec<_>>();
            draw_lines(f, area, &chats, users, true);
        }
        Some(Offset::Absolute(offset)) => {
            let chats = chats.skip(offset as usize).take(height).collect::<Vec<_>>();
            draw_lines(f, area, &chats, users, false);
        }
    }
}

/// processes the events into lines and draws them to the passed in `Frame`,
/// the events may or may not fit onto the screen
fn draw_lines(
    f: &'_ mut Frame,
    area: Rect,
    events: &[&RoomEvent],
    users: &impl UserLocator,
    prioritize_last: bool,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let row_areas = area.rows().collect::<Vec<_>>();
    let mut rows = Vec::<Line>::with_capacity(area.height as usize);
    let area_width = area.width as usize;
    let area_height = area.height as usize;

    let event_props = events
        .iter()
        .map(|ev| ev.properties(users))
        .collect::<Vec<_>>();

    let max_user_width = 1 + event_props
        .iter()
        .filter_map(|p| match p {
            EventType::Info { .. } => None,
            EventType::User(user_event_properties) => Some(user_event_properties.user_width()),
        })
        .max()
        .unwrap_or_default();

    if max_user_width > area_width {
        let chars = "Width too small".chars().collect::<Vec<_>>();
        for chars in chars.chunks(area.width as usize) {
            rows.push(Line::from(String::from_iter(chars)));
        }
    }

    for props in &event_props {
        match props {
            EventType::Info { message, style } => {
                let message_characters = message.chars().collect::<Vec<_>>();
                // split the message into collections of characters that fit in the given area
                for msg in message_characters.chunks(area_width) {
                    rows.push(Line::from(Span::from(String::from_iter(msg)).style(*style)));
                }
            }
            EventType::User(user_event) => {
                let event_rows = user_event.build_lines(max_user_width, area_width);
                rows.extend(event_rows);
            }
        }
    }

    if prioritize_last {
        for (r, a) in rows.iter().rev().take(area_height).rev().zip(row_areas) {
            f.render_widget(r, a);
        }
    } else {
        for (r, a) in rows.iter().zip(row_areas) {
            f.render_widget(r, a);
        }
    }
}
