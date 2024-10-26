use crate::render::LineWithRenderScheme;

use std::io::{stdout, Write};

use anyhow::{Ok, Result};
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

#[derive(Debug, Clone, Default)]
pub struct Canvas {
    pub body_area: Vec<LineWithRenderScheme>,
    pub popup_menu: Vec<LineWithRenderScheme>,
    pub status_bar: LineWithRenderScheme,
}

impl Canvas {
    pub fn clear(&mut self) {
        self.body_area.clear();
        self.popup_menu.clear();
        self.status_bar.clear();
    }

    pub fn render(&self) -> Result<()> {
        let body_area_height = self.body_area.len() - self.popup_menu.len();
        clear_screen_and_reset_cursor()?;
        for line in self.body_area.iter().take(body_area_height) {
            let line = line.render();
            println!("{line}\r");
        }

        for line in self.popup_menu.iter() {
            let line = line.render();
            println!("{line}\r");
        }

        print!("{}", self.status_bar.render());
        stdout().flush().unwrap();
        Ok(())
    }
}

pub fn clear_screen_and_reset_cursor() -> Result<()> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}
