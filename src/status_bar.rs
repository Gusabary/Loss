#[derive(Debug, Default)]
pub struct StatusBar {
    text: String,
    oneoff_error_text: Option<String>,
}

impl StatusBar {
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn clear_text(&mut self) {
        self.text = String::default();
    }

    pub fn set_oneoff_error_text(&mut self, text: &str) {
        self.oneoff_error_text = Some(text.to_string());
    }

    pub fn render_text(&mut self, width: usize, ratio: usize) -> String {
        if let Some(mut text) = self.oneoff_error_text.clone() {
            self.oneoff_error_text = None;
            text.truncate(width);
            return text;
        }
        let mut text = self.text.clone();
        if self.text.len() + 6 < width {
            let ratio_str = format!("{ratio}%");
            assert!(ratio_str.len() <= 4);
            let space_count = width - self.text.len() - ratio_str.len();
            text.extend(std::iter::repeat(' ').take(space_count));
            text.push_str(&ratio_str);
        } else {
            text.truncate(width);
        }
        text
    }
}
