use std::time::Duration;

use anyhow::{Ok, Result};
use crossterm::event::{self, poll, read, KeyCode, KeyEvent, KeyModifiers};
use log::info;

use crate::{
    bookmark::{BookMarkMenu, BookmarkMenuAction},
    finder::{FinderAction, FinderEventParser},
    prompt::{Prompt, PromptAction},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    #[allow(dead_code)]
    pub fn is_horizontal(&self) -> bool {
        *self == Direction::Left || *self == Direction::Right
    }

    pub fn is_vertical(&self) -> bool {
        *self == Direction::Up || *self == Direction::Down
    }

    pub fn above_or_below(&self) -> &str {
        assert!(self.is_vertical());
        if *self == Direction::Up {
            "above"
        } else {
            "below"
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum InterruptState {
    #[default]
    Uninterruptable,
    Interruptable,
    Interrupted,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    WindowMove(Direction, usize),
    Exit,
    ToggleWrapLine,
    Search(PromptAction),
    SearchNext,
    SearchPrevious,
    SeekToHome,
    SeekToEnd,
    JumpToTimestamp(PromptAction),
    JumpByLines(PromptAction),
    TerminalResize(usize, usize),
    NewBookmark(PromptAction),
    GotoBookmark(BookmarkMenuAction),
    UndoWindowVerticalMove,
    RedoWindowVerticalMove,
    FinderOperation(FinderAction),
}

#[derive(Debug, Default)]
pub struct EventSource {
    search_prompt: Prompt,
    timestamp_prompt: Prompt,
    jump_prompt: Prompt,
    new_bookmark_prompt: Prompt,
    bookmark_menu: BookMarkMenu,
    finder_event_parser: FinderEventParser,
}

impl EventSource {
    pub fn check_for_interrupt(&mut self) -> Result<bool> {
        let has_event = poll(Duration::from_secs(0))?;
        if has_event {
            let raw_event = read()?;
            if let event::Event::Key(key) = raw_event {
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        loop {
            let raw_event = read()?;
            let event = self.handle_raw_event(&raw_event);
            if let Some(event) = event {
                return Ok(event);
            }
        }
    }

    fn handle_raw_event(&mut self, raw_event: &event::Event) -> Option<Event> {
        info!("raw event: {:?}", raw_event);
        match raw_event {
            event::Event::Key(key) => self.handle_key_press(key),
            event::Event::Resize(width, height) => {
                Some(Event::TerminalResize(*width as usize, *height as usize))
            }
            _ => None,
        }
    }

    fn handle_key_press(&mut self, key: &KeyEvent) -> Option<Event> {
        if self.search_prompt.is_active() {
            return self.search_prompt.handle_raw_event(key).map(Event::Search);
        }
        if self.timestamp_prompt.is_active() {
            return self
                .timestamp_prompt
                .handle_raw_event(key)
                .map(Event::JumpToTimestamp);
        }
        if self.jump_prompt.is_active() {
            return self
                .jump_prompt
                .handle_raw_event(key)
                .map(Event::JumpByLines);
        }
        if self.new_bookmark_prompt.is_active() {
            return self
                .new_bookmark_prompt
                .handle_raw_event(key)
                .map(Event::NewBookmark);
        }
        if self.bookmark_menu.is_active() {
            return self
                .bookmark_menu
                .handle_raw_event(key)
                .map(Event::GotoBookmark);
        }
        if let Some(action) = self.finder_event_parser.try_parse_raw_event(key) {
            return Some(Event::FinderOperation(action));
        }

        if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT {
            match key.code {
                KeyCode::Char('q') => Some(Event::Exit),
                KeyCode::Char('w') => Some(Event::ToggleWrapLine),
                KeyCode::Char('/') => {
                    self.search_prompt.start();
                    Some(Event::Search(PromptAction::Start(Some(Direction::Down))))
                }
                KeyCode::Char('?') => {
                    self.search_prompt.start();
                    Some(Event::Search(PromptAction::Start(Some(Direction::Up))))
                }
                KeyCode::Char('t') => {
                    self.timestamp_prompt.start();
                    Some(Event::JumpToTimestamp(PromptAction::Start(None)))
                }
                KeyCode::Char('n') => Some(Event::SearchNext),
                KeyCode::Char('N') => Some(Event::SearchPrevious),
                KeyCode::Down => Some(Event::WindowMove(Direction::Down, 1)),
                KeyCode::Up => Some(Event::WindowMove(Direction::Up, 1)),
                KeyCode::Right => Some(Event::WindowMove(Direction::Right, 1)),
                KeyCode::Left => Some(Event::WindowMove(Direction::Left, 1)),
                KeyCode::PageDown => Some(Event::WindowMove(Direction::Down, 5)),
                KeyCode::PageUp => Some(Event::WindowMove(Direction::Up, 5)),
                KeyCode::Home => Some(Event::SeekToHome),
                KeyCode::End => Some(Event::SeekToEnd),
                KeyCode::Char('j') => {
                    self.jump_prompt.start();
                    Some(Event::JumpByLines(PromptAction::Start(Some(
                        Direction::Down,
                    ))))
                }
                KeyCode::Char('J') => {
                    self.jump_prompt.start();
                    Some(Event::JumpByLines(PromptAction::Start(Some(Direction::Up))))
                }
                KeyCode::Char('b') => {
                    self.new_bookmark_prompt.start();
                    Some(Event::NewBookmark(PromptAction::Start(None)))
                }
                KeyCode::Char('g') => {
                    self.bookmark_menu.activate();
                    Some(Event::GotoBookmark(BookmarkMenuAction::Start))
                }
                KeyCode::Char(',') => Some(Event::UndoWindowVerticalMove),
                KeyCode::Char('.') => Some(Event::RedoWindowVerticalMove),
                _ => None,
            }
        } else if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                KeyCode::Down => Some(Event::WindowMove(Direction::Down, 5)),
                KeyCode::Up => Some(Event::WindowMove(Direction::Up, 5)),
                KeyCode::PageDown => Some(Event::WindowMove(Direction::Down, 20)),
                KeyCode::PageUp => Some(Event::WindowMove(Direction::Up, 20)),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use event::Event as RawEvent;

    #[test]
    fn test_search_event() {
        let mut source = EventSource::default();
        assert_eq!(
            source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                KeyCode::Char('/'),
                KeyModifiers::NONE
            ))),
            Some(Event::Search(PromptAction::Start(Some(Direction::Down))))
        );
        assert_eq!(
            source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                KeyCode::Char('1'),
                KeyModifiers::NONE
            ))),
            Some(Event::Search(PromptAction::Content("1".to_string())))
        );
        assert_eq!(
            source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                KeyCode::Backspace,
                KeyModifiers::NONE
            ))),
            Some(Event::Search(PromptAction::Content("".to_string())))
        );
        assert_eq!(
            source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE
            ))),
            Some(Event::Search(PromptAction::Cancel))
        );
        assert_eq!(
            source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                KeyCode::Char('?'),
                KeyModifiers::NONE
            ))),
            Some(Event::Search(PromptAction::Start(Some(Direction::Up))))
        );
        let mut content = String::default();
        for c in 'a'..='c' {
            content.push(c);
            assert_eq!(
                source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                    KeyCode::Char(c),
                    KeyModifiers::NONE
                ))),
                Some(Event::Search(PromptAction::Content(content.clone())))
            );
        }
        assert_eq!(
            source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE
            ))),
            Some(Event::Search(PromptAction::Enter("abc".to_string())))
        );
    }

    #[test]
    fn test_window_move_event() {
        let mut source = EventSource::default();
        assert_eq!(
            source.handle_raw_event(&RawEvent::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::NONE
            ))),
            Some(Event::WindowMove(Direction::Down, 1))
        );
    }
}
