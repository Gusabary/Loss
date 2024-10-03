use crate::event_source::Direction;

#[derive(Debug, PartialEq)]
pub enum PromptAction {
    Start(Option<Direction>),
    Content(String),
    Enter(String),
    Cancel,
}

#[derive(Debug, Default)]
pub struct PromptHistory {
    prompts: Vec<String>,
    current_index: usize,
}

impl PromptHistory {
    pub fn push(&mut self, prompt: &str) {
        if !self.prompts.is_empty() && self.prompts.last().unwrap() == prompt {
            return;
        }
        self.prompts.push(prompt.to_string());
    }

    pub fn reset_index(&mut self) {
        self.current_index = self.prompts.len();
    }

    pub fn previous_one(&mut self) -> String {
        self.current_index = self.current_index.saturating_sub(1);
        self.current_one()
    }

    pub fn next_one(&mut self) -> String {
        self.current_index = std::cmp::min(self.current_index + 1, self.prompts.len());
        self.current_one()
    }

    fn current_one(&self) -> String {
        if self.current_index < self.prompts.len() {
            self.prompts[self.current_index].clone()
        } else {
            String::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_history() {
        let mut history = PromptHistory::default();
        history.reset_index();
        assert_eq!(history.previous_one(), String::default());
        assert_eq!(history.next_one(), String::default());

        history.push("123");
        history.reset_index();
        assert_eq!(history.previous_one(), "123".to_string());
        assert_eq!(history.next_one(), String::default());
        assert_eq!(history.previous_one(), "123".to_string());

        history.push("456");
        history.reset_index();
        assert_eq!(history.previous_one(), "456".to_string());
        assert_eq!(history.previous_one(), "123".to_string());
        assert_eq!(history.previous_one(), "123".to_string());
        assert_eq!(history.next_one(), "456".to_string());
        assert_eq!(history.next_one(), String::default());

        history.push("456");
        assert_eq!(history.prompts.len(), 2);

        history.push("123");
        assert_eq!(history.prompts.len(), 3);
    }
}
