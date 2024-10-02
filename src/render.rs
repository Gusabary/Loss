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
    pub bottom_line_text: String,
    pub oneoff_bottom_line_text: Option<String>,
}

impl Renderer {
    pub fn render(&mut self) -> Result<()> {
        let render_buffer = self
            .buffer
            .iter()
            .map(|row| self.render_line(row))
            .collect::<Vec<_>>();

        clear_screen_and_reset_cursor()?;
        for line in render_buffer {
            println!("{line}\r");
        }

        if let Some(text) = &self.oneoff_bottom_line_text {
            print!("{}", text);
            self.oneoff_bottom_line_text = None;
        } else {
            print!("{}", self.bottom_line_text);
        }
        stdout().flush().unwrap();
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
        format!("{line}")
    }
}

pub fn clear_screen_and_reset_cursor() -> Result<()> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}
