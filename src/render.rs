use std::io::stdout;

use anyhow::{Ok, Result};
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

#[derive(Debug, Default)]
pub struct RenderOptions {
    pub wrap_lines: bool,
}

#[derive(Debug, Default)]
pub struct Renderer {
    pub buffer: Vec<String>,
    pub options: RenderOptions,
}

impl Renderer {
    pub fn render(&self) -> Result<()> {
        let render_buffer = self
            .buffer
            .iter()
            .map(|row| self.render_line(row))
            .collect::<Vec<_>>();

        clear_screen_and_reset_cursor()?;
        for line in render_buffer {
            println!("{line}\r");
        }

        Ok(())
    }

    fn render_line(&self, line: &str) -> String {
        format!("{line}")
    }
}

fn clear_screen_and_reset_cursor() -> Result<()> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}
