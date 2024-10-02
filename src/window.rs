use anyhow::{Ok, Result};
use crossterm::terminal;

use crate::event_source::Direction;

#[derive(Debug)]
pub struct Window {
    pub height: usize,
    pub width: usize,
    // offset of first line start instead of top-left corner of window
    pub offset: usize,
    pub horizontal_shift: usize,
}

impl Window {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            height: height as usize - 1,
            width: width as usize,
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
}
