use anyhow::{Ok, Result};
use crossterm::terminal;

use crate::event_source::Direction;

#[derive(Debug)]
pub struct Window {
    pub width: usize,
    pub height: usize,
    // offset of first line start instead of top-left corner of window
    pub offset: usize,
    pub horizontal_shift: usize,
}

impl Window {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            width: width as usize,
            height: height as usize - 1,
            offset: 0,
            horizontal_shift: 0,
        })
    }

    pub fn move_offset_by(&mut self, distance: usize, direction: Direction) {
        assert!(direction.is_vertical());
        if direction == Direction::Up {
            self.offset = self.offset.saturating_sub(distance);
        } else {
            self.offset += distance;
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height - 1;
    }
}
