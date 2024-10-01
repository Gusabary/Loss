use anyhow::{Ok, Result};
use crossterm::event::{self, read, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    WindowMove(Direction, usize),
    Exit,
}

pub struct EventHub {}

impl EventHub {
    pub fn wait_for_event(&self) -> Result<Event> {
        loop {
            let raw_event = read()?;
            let event = match raw_event {
                event::Event::Key(key) => self.handle_key_press(&key)?,
                _ => None,
            };
            if event.is_some() {
                return Ok(event.unwrap());
            }
        }
    }

    fn handle_key_press(&self, key: &KeyEvent) -> Result<Option<Event>> {
        if key.modifiers == KeyModifiers::NONE {
            match key.code {
                KeyCode::Char('q') => Ok(Some(Event::Exit)),
                KeyCode::Down => Ok(Some(Event::WindowMove(Direction::Down, 1))),
                KeyCode::Up => Ok(Some(Event::WindowMove(Direction::Up, 1))),
                KeyCode::Right => Ok(Some(Event::WindowMove(Direction::Right, 1))),
                KeyCode::Left => Ok(Some(Event::WindowMove(Direction::Left, 1))),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}
