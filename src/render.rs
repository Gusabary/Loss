use std::{ops::Range, vec};

use crossterm::style::Stylize;

use crate::finder::HighlightOption;

#[derive(Debug, Copy, Clone)]
pub enum RenderScheme {
    Dim,
    Highlight(HighlightOption),
}

#[derive(Debug, Clone, Default)]
pub struct LineWithRenderScheme {
    content: String,
    render_schemes: Vec<(Range<usize>, RenderScheme)>,
}

impl LineWithRenderScheme {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            render_schemes: vec![],
        }
    }

    pub fn truncate(mut self, width: usize) -> Self {
        self.content.truncate(width);
        self
    }

    pub fn raw_content(&self) -> &str {
        &self.content
    }

    pub fn set_raw_content(&mut self, raw_content: &str) {
        self.content = raw_content.to_string();
    }

    pub fn add_scheme_if_not_overlap(&mut self, range: Range<usize>, scheme: RenderScheme) {
        if self
            .render_schemes
            .iter()
            .all(|(existing_range, _)| !ranges_have_overlap(range.clone(), existing_range.clone()))
        {
            self.render_schemes.push((range, scheme));
        }
    }

    pub fn substr(&self, width_range: Range<usize>) -> LineWithRenderScheme {
        let content = if width_range.start >= self.content.len() {
            String::default()
        } else {
            let end = std::cmp::min(width_range.end, self.content.len());
            self.content[width_range.start..end].to_string()
        };
        let mut sub_schemes = vec![];
        for (range, scheme) in self.render_schemes.iter() {
            let new_start = std::cmp::max(range.start, width_range.start);
            let new_end = std::cmp::min(range.end, width_range.end);
            if new_start < new_end {
                let s = new_start - width_range.start;
                let e = new_end - width_range.start;
                sub_schemes.push((s..e, *scheme));
            }
        }
        LineWithRenderScheme {
            content,
            render_schemes: sub_schemes,
        }
    }

    pub fn render(&self) -> String {
        let mut render_schemes = self.render_schemes.clone();
        render_schemes.sort_by(|a, b| a.0.start.cmp(&b.0.start));
        for window in render_schemes.windows(2) {
            assert!(window[0].0.end <= window[1].0.start);
        }
        let mut rendered_line = self.content.to_string();
        for (range, scheme) in render_schemes.into_iter().rev() {
            let raw = self.content[range.clone()].to_string();
            let rendered = match scheme {
                RenderScheme::Dim => raw.dim().to_string(),
                RenderScheme::Highlight(option) => option.render(&raw),
            };
            rendered_line.replace_range(range, &rendered);
        }
        rendered_line
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.render_schemes.clear();
    }
}

fn ranges_have_overlap(r1: Range<usize>, r2: Range<usize>) -> bool {
    r1.start < r2.end && r1.end > r2.start
}
