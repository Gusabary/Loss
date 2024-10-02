use anyhow::{Ok, Result};
use crossterm::event::{self, read, KeyCode, KeyEvent, KeyModifiers};
use log::info;

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
}

#[derive(Debug, PartialEq)]
pub enum PromptAction {
    Start(Direction),
    Content(String),
    Enter(String),
    Cancel,
    // todo: Direction for history
}

#[derive(Debug, PartialEq)]
pub enum Event {
    WindowMove(Direction, usize),
    Exit,
    ToggleWrapLine,
    Search(PromptAction),
    // todo: maybe aggregate to a Jump event ?
    Next,
    Previous,
    SeekToHome,
    SeekToEnd,
}

#[derive(Debug, Default)]
pub struct EventSource {
    search_prompt: Option<String>,
}

impl EventSource {
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
            _ => None,
        }
    }

    fn handle_key_press(&mut self, key: &KeyEvent) -> Option<Event> {
        if self.search_prompt.is_some() {
            let action = self.handle_search_prompt(key);
            if let Some(action) = action {
                return Some(Event::Search(action));
            } else {
                return None;
            }
        }
        if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT {
            match key.code {
                KeyCode::Char('q') => Some(Event::Exit),
                KeyCode::Char('w') => Some(Event::ToggleWrapLine),
                KeyCode::Char('/') => {
                    self.search_prompt = Some(String::default());
                    Some(Event::Search(PromptAction::Start(Direction::Down)))
                }
                KeyCode::Char('?') => {
                    self.search_prompt = Some(String::default());
                    Some(Event::Search(PromptAction::Start(Direction::Up)))
                }
                KeyCode::Char('n') => Some(Event::Next),
                KeyCode::Char('N') => Some(Event::Previous),
                KeyCode::Down => Some(Event::WindowMove(Direction::Down, 1)),
                KeyCode::Up => Some(Event::WindowMove(Direction::Up, 1)),
                KeyCode::Right => Some(Event::WindowMove(Direction::Right, 1)),
                KeyCode::Left => Some(Event::WindowMove(Direction::Left, 1)),
                KeyCode::PageDown => Some(Event::WindowMove(Direction::Down, 5)),
                KeyCode::PageUp => Some(Event::WindowMove(Direction::Up, 5)),
                KeyCode::Home => Some(Event::SeekToHome),
                KeyCode::End => Some(Event::SeekToEnd),
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

    fn handle_search_prompt(&mut self, key: &KeyEvent) -> Option<PromptAction> {
        assert!(self.search_prompt.is_some());
        let search_prompt = self.search_prompt.as_mut().unwrap();
        if key.modifiers != KeyModifiers::NONE && key.modifiers != KeyModifiers::SHIFT {
            None
        } else {
            match key.code {
                KeyCode::Char(c) => {
                    search_prompt.push(c);
                    Some(PromptAction::Content(search_prompt.clone()))
                }
                KeyCode::Backspace => {
                    search_prompt.pop();
                    Some(PromptAction::Content(search_prompt.clone()))
                }
                KeyCode::Enter => {
                    let prompt = search_prompt.clone();
                    self.search_prompt = None;
                    Some(PromptAction::Enter(prompt))
                }
                KeyCode::Esc => {
                    self.search_prompt = None;
                    Some(PromptAction::Cancel)
                }
                _ => None,
            }
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
            Some(Event::Search(PromptAction::Start(Direction::Down)))
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
            Some(Event::Search(PromptAction::Start(Direction::Up)))
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
