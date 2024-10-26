use crate::{canvas::Canvas, render::LineWithRenderScheme};

#[derive(Debug, Default)]
pub struct StatusBar {
    text: String,
    oneoff_error_text: Option<String>,
    ratio: usize,
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

    pub fn set_ratio(&mut self, ratio: usize) {
        self.ratio = ratio;
    }

    pub fn render(&mut self, canvas: &mut Canvas, window_width: usize) -> Option<usize> {
        if let Some(text) = self.oneoff_error_text.clone() {
            self.oneoff_error_text = None;
            canvas.status_bar = LineWithRenderScheme::new(&text).truncate(window_width);
            canvas.cursor_pos_x = Some(text.len());
            return None;
        }
        let mut text = self.text.clone();
        canvas.cursor_pos_x = Some(text.len());
        let space_count;
        if self.text.len() + 6 < window_width {
            let ratio_str = format!("{}%", self.ratio);
            assert!(ratio_str.len() <= 4);
            space_count = Some(window_width - self.text.len() - ratio_str.len());
            text.extend(std::iter::repeat(' ').take(space_count.unwrap()));
            text.push_str(&ratio_str);
        } else {
            space_count = None;
            text.truncate(window_width);
        }
        canvas.status_bar = LineWithRenderScheme::new(&text);
        space_count
    }
}
