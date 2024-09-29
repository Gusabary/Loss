use std::io::stdout;

use anyhow::{Ok, Result};
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

use crate::manager::WindowSize;

pub struct RenderOptions {}

pub fn render(
    lines: &Vec<String>,
    window_size: &WindowSize,
    render_options: &RenderOptions,
) -> Result<()> {
    for (index, line) in lines.iter().enumerate() {
        println!("{line}");
    }

    let displayable_lines = lines;
    let render_buffer = displayable_lines
        .iter()
        .map(|row| render_line(row))
        .collect::<Vec<_>>();

    clear_screen_and_reset_cursor()?;
    for line in render_buffer {
        println!("{line}\r");
    }
    Ok(())
}

fn render_line(line: &str) -> String {
    format!("{line}")
}

fn clear_screen_and_reset_cursor() -> Result<()> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}
