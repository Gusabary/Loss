use std::{
    io::{stdout, Write},
    ops::Range,
};

use anyhow::{Ok, Result};
use crossterm::{
    cursor::MoveTo,
    style::{Color, Stylize},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

#[derive(Debug, Default)]
pub struct Renderer {
    pub buffer: Vec<String>,
    pub popup_menu_render_text: Vec<String>,
    pub status_bar_render_text: String,
}

impl Renderer {
    pub fn render(&mut self) -> Result<()> {
        clear_screen_and_reset_cursor()?;
        for line in self.buffer.iter() {
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
}

pub fn clear_screen_and_reset_cursor() -> Result<()> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}

pub enum RenderScheme {
    Dim,
    ForegroundColor(Color),
    BackgroundColor(Color),
}

pub fn render_line(line: &str, mut range_scheme: Vec<(Range<usize>, Vec<RenderScheme>)>) -> String {
    range_scheme.sort_by(|a, b| a.0.start.cmp(&b.0.start));
    for window in range_scheme.windows(2) {
        assert!(window[0].0.end <= window[1].0.start);
    }
    let mut rendered_line = line.to_string();
    for (range, schemes) in range_scheme.into_iter().rev() {
        let mut rendered_part = line[range.clone()].to_string();
        for scheme in schemes {
            rendered_part = match scheme {
                RenderScheme::Dim => rendered_part.dim(),
                RenderScheme::ForegroundColor(color) => rendered_part.with(color),
                RenderScheme::BackgroundColor(color) => rendered_part.on(color),
            }
            .to_string();
        }
        rendered_line.replace_range(range, &rendered_part);
    }
    rendered_line
}
