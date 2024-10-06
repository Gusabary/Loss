use crate::render::Renderer;

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

    pub fn render(&mut self, renderer: &mut Renderer, window_width: usize) {
        if let Some(mut text) = self.oneoff_error_text.clone() {
            self.oneoff_error_text = None;
            text.truncate(window_width);
            renderer.status_bar_render_text = text;
            return;
        }
        let mut text = self.text.clone();
        if self.text.len() + 6 < window_width {
            let ratio_str = format!("{}%", self.ratio);
            assert!(ratio_str.len() <= 4);
            let space_count = window_width - self.text.len() - ratio_str.len();
            text.extend(std::iter::repeat(' ').take(space_count));
            text.push_str(&ratio_str);
        } else {
            text.truncate(window_width);
        }
        renderer.status_bar_render_text = text;
    }
}
