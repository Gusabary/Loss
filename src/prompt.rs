use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::event_source::Direction;

#[derive(Debug, PartialEq)]
pub enum PromptAction {
    Start(Option<Direction>),
    Content(String),
    Enter(String),
    Cancel,
}

#[derive(Debug, Default)]
pub struct Prompt {
    prompt_text: Option<String>,
    prompt_history: Vec<String>,
    history_index: usize,
}

impl Prompt {
    pub fn start(&mut self) {
        self.prompt_text = Some(String::default());
        self.history_index = self.prompt_history.len();
    }

    fn finish(&mut self) {
        self.prompt_text = None;
    }

    pub fn is_active(&self) -> bool {
        self.prompt_text.is_some()
    }

    fn push_history(&mut self, prompt: &str) {
        if !self.prompt_history.is_empty() && self.prompt_history.last().unwrap() == prompt {
            return;
        }
        self.prompt_history.push(prompt.to_string());
    }

    fn previous_one(&mut self) -> String {
        self.history_index = self.history_index.saturating_sub(1);
        self.current_one()
    }

    fn next_one(&mut self) -> String {
        self.history_index = std::cmp::min(self.history_index + 1, self.prompt_history.len());
        self.current_one()
    }

    fn current_one(&self) -> String {
        if self.history_index < self.prompt_history.len() {
            self.prompt_history[self.history_index].clone()
        } else {
            String::default()
        }
    }

    pub fn handle_raw_event(&mut self, key: &KeyEvent) -> Option<PromptAction> {
        assert!(self.is_active());
        if key.modifiers != KeyModifiers::NONE && key.modifiers != KeyModifiers::SHIFT {
            None
        } else {
            let prompt_text = self.prompt_text.as_mut().unwrap();
            match key.code {
                KeyCode::Char(c) => {
                    prompt_text.push(c);
                    Some(PromptAction::Content(prompt_text.to_string()))
                }
                KeyCode::Backspace => {
                    prompt_text.pop();
                    Some(PromptAction::Content(prompt_text.to_string()))
                }
                KeyCode::Enter => {
                    let prompt_text = prompt_text.clone();
                    self.push_history(&prompt_text);
                    self.finish();
                    Some(PromptAction::Enter(prompt_text))
                }
                KeyCode::Esc => {
                    self.finish();
                    Some(PromptAction::Cancel)
                }
                KeyCode::Up => {
                    self.prompt_text = Some(self.previous_one());
                    Some(PromptAction::Content(
                        self.prompt_text.as_ref().unwrap().to_string(),
                    ))
                }
                KeyCode::Down => {
                    self.prompt_text = Some(self.next_one());
                    Some(PromptAction::Content(
                        self.prompt_text.as_ref().unwrap().to_string(),
                    ))
                }
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_history() {
        let mut prompt = Prompt::default();
        prompt.start();
        assert_eq!(prompt.previous_one(), String::default());
        assert_eq!(prompt.next_one(), String::default());

        prompt.push_history("123");
        prompt.start();
        assert_eq!(prompt.previous_one(), "123".to_string());
        assert_eq!(prompt.next_one(), String::default());
        assert_eq!(prompt.previous_one(), "123".to_string());

        prompt.push_history("456");
        prompt.start();
        assert_eq!(prompt.previous_one(), "456".to_string());
        assert_eq!(prompt.previous_one(), "123".to_string());
        assert_eq!(prompt.previous_one(), "123".to_string());
        assert_eq!(prompt.next_one(), "456".to_string());
        assert_eq!(prompt.next_one(), String::default());

        prompt.push_history("456");
        assert_eq!(prompt.prompt_history.len(), 2);

        prompt.push_history("123");
        assert_eq!(prompt.prompt_history.len(), 3);
    }
}
