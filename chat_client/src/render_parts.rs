use std::num::NonZero;

use chat_lib::types::User;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::room_event::RoomEvent;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Offset {
    /// offset from the top
    Absolute(u32),
    /// offset relative to the most recent chat
    Relative(NonZero<u32>),
}

pub fn top_block<'a>() -> Block<'a> {
    Block::new().borders(Borders::BOTTOM)
}

pub fn draw_top_bar<N>(f: &'_ mut Frame, area: Rect, name: N)
where
    N: AsRef<str>,
{
    let msg = "Press Ctrl+h to display help menu";
    let name = name.as_ref().to_string();
    let spaces =
        area.width as i32 - (name.chars().count() as i32) - 1 - (msg.chars().count() as i32);
    let spaces = spaces.max(1);
    let text = name + " ".repeat(spaces as usize).as_str() + msg;
    let para = Paragraph::new(text).block(top_block());

    f.render_widget(para, area);
}

pub fn draw_room_events<'a, C>(
    f: &'_ mut Frame,
    area: Rect,
    chats: C,
    users: &Vec<User>,
    offset: Option<Offset>,
) where
    C: DoubleEndedIterator<Item = &'a RoomEvent> + ExactSizeIterator,
{
    let height = area.height as usize;
    match offset {
        None => {
            let chats = chats.rev().take(height).rev();
            draw_lines(f, area, chats, users);
        }
        Some(Offset::Absolute(offset)) => {
            let chats = chats.skip(offset as usize).take(height);
            draw_lines(f, area, chats, users);
        }
        Some(Offset::Relative(offset)) => {
            let chats = chats.rev().skip(offset.get() as usize).take(height);
            draw_lines(f, area, chats, users);
        }
    }
}

fn draw_lines<'a, C>(f: &'_ mut Frame, area: Rect, chats: C, users: &Vec<User>)
where
    C: Iterator<Item = &'a RoomEvent>,
{
    for (c, r) in chats.zip(area.rows()) {
        let row = c.as_line(area.width, users);
        f.render_widget(row, r);
    }
}
