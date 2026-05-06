use std::num::NonZero;

use chat_lib::types::User;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::room_event::{EventProperties, RoomEvent};

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

pub fn draw_top_bar<N>(f: &'_ mut Frame, area: Rect, name: N)
where
    N: AsRef<str>,
{
    let msg = "Press Ctrl+h to display help menu";
    let name = name.as_ref().to_string();
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let spaces =
        i32::from(area.width) - (name.chars().count() as i32) - 1 - (msg.chars().count() as i32);
    #[allow(clippy::cast_sign_loss)]
    let spaces = spaces.max(1) as u32;
    let text = name + " ".repeat(spaces as usize).as_str() + msg;
    let para = Paragraph::new(text).block(top_block());

    f.render_widget(para, area);
}

pub fn draw_room_events<'a, C>(
    f: &'_ mut Frame,
    area: Rect,
    chats: C,
    users: &[User],
    offset: Option<Offset>,
) where
    C: DoubleEndedIterator<Item = &'a RoomEvent> + ExactSizeIterator,
{
    let height = area.height as usize;
    match offset {
        None => {
            let chats = chats.rev().take(height).rev().collect::<Vec<_>>();
            draw_lines(f, area, &chats, users);
        }
        Some(Offset::Absolute(offset)) => {
            let chats = chats.skip(offset as usize).take(height).collect::<Vec<_>>();
            draw_lines(f, area, &chats, users);
        }
        Some(Offset::Relative(offset)) => {
            let chats = chats
                .rev()
                .skip(offset.get() as usize)
                .take(height)
                .rev()
                .collect::<Vec<_>>();
            draw_lines(f, area, &chats, users);
        }
    }
}

/// processes the events into lines and draws them to the passed in `Frame`,
/// the events may or may not fit onto the screen
fn draw_lines(f: &'_ mut Frame, area: Rect, events: &[&RoomEvent], users: &[User]) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let row_areas = area.rows().collect::<Vec<_>>();
    let mut rows = Vec::<Line>::with_capacity(area.height as usize);
    let area_width = area.width as usize;

    let event_props = events
        .iter()
        .map(|ev| ev.properties(users))
        .collect::<Vec<_>>();

    let max_user_width = 1 + event_props
        .iter()
        .filter_map(|p| match p {
            EventProperties::Info { .. } => None,
            EventProperties::User(user_event_properties) => {
                Some(user_event_properties.user_width())
            }
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
            EventProperties::Info { message, style } => {
                let message_characters = message.chars().collect::<Vec<_>>();
                // split the message into collections of characters that fit in the given area
                for msg in message_characters.chunks(area_width) {
                    rows.push(Line::from(Span::from(String::from_iter(msg)).style(*style)));
                }
            }
            EventProperties::User(user_event) => {
                let event_rows = user_event.build_lines(max_user_width, area_width);
                rows.extend(event_rows);
            }
        }
    }

    for (r, a) in rows.iter().zip(row_areas) {
        f.render_widget(r, a);
    }
}
