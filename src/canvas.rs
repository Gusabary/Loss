use crate::render::LineWithRenderScheme;

use std::io::{stdout, Write};

use anyhow::{Ok, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

#[derive(Debug, Clone, Default)]
pub struct Canvas {
    pub body_area: Vec<LineWithRenderScheme>,
    pub popup_menu: Vec<LineWithRenderScheme>,
    pub status_bar: LineWithRenderScheme,
    pub cursor_pos_x: Option<usize>,
}

impl Canvas {
    pub fn clear(&mut self) {
        self.body_area.clear();
        self.popup_menu.clear();
        self.status_bar.clear();
    }

    pub fn render(&self) -> Result<()> {
        let mut screen_buffer: Vec<String> = vec![];
        let body_area_height = self.body_area.len() - self.popup_menu.len();
        for line in self.body_area.iter().take(body_area_height) {
            screen_buffer.push(format!("{}\r\n", line.render()));
        }
        for line in self.popup_menu.iter() {
            screen_buffer.push(format!("{}\r\n", line.render()));
        }
        screen_buffer.push(self.status_bar.render());

        clear_screen_and_reset_cursor()?;
        for line in screen_buffer {
            print!("{line}");
        }
        stdout().flush().unwrap();

        if let Some(x) = self.cursor_pos_x {
            stdout()
                .execute(Show)?
                .execute(MoveTo(x as u16, self.body_area.len() as u16))?;
        } else {
            stdout().execute(Hide)?;
        }

        Ok(())
    }
}

pub fn clear_screen_and_reset_cursor() -> Result<()> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}
