use std::io::{stdout, Write};

use anyhow::{Ok, Result};
use crossterm::{
    cursor::MoveTo,
    style::{Color, Stylize},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

#[derive(Debug, Default)]
pub struct RenderOptions {
    pub wrap_lines: bool,
    pub highlight_text: Option<String>,
}

#[derive(Debug, Default)]
pub struct Renderer {
    pub buffer: Vec<String>,
    pub options: RenderOptions,
    pub popup_menu_render_text: Vec<String>,
    pub status_bar_render_text: String,
}

impl Renderer {
    pub fn render(&mut self) -> Result<()> {
        let render_buffer = self
            .buffer
            .iter()
            .take(self.buffer.len() - self.popup_menu_render_text.len())
            .map(|row| self.render_line(row))
            .collect::<Vec<_>>();

        clear_screen_and_reset_cursor()?;
        for line in render_buffer {
            println!("{line}\r");
        }

        for line in self.popup_menu_render_text.iter() {
            println!("{line}\r");
        }

        print!("{}", self.status_bar_render_text);
        stdout().flush().unwrap();

        self.popup_menu_render_text.clear();

        Ok(())
    }

    fn render_line(&self, line: &str) -> String {
        if let Some(text) = &self.options.highlight_text {
            if let Some(index) = line.find(text) {
                let end = index + text.len();
                let styled = line[index..end].with(Color::Black).on(Color::Grey);
                return format!("{}{}{}\r", &line[..index], styled, &line[end..]);
            }
        }
        line.to_string()
    }
}

pub fn clear_screen_and_reset_cursor() -> Result<()> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}
