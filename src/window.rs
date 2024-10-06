use anyhow::{Ok, Result};
use crossterm::terminal;

use crate::event_source::Direction;

#[derive(Debug)]
struct OffsetHistory {
    offsets: Vec<usize>,
    current_index: usize,
}

impl OffsetHistory {
    fn new() -> Self {
        Self {
            offsets: vec![0],
            current_index: 0,
        }
    }

    fn push(&mut self, offset: usize) {
        assert!(self.offsets.len() > self.current_index);
        self.offsets.truncate(self.current_index + 1);
        self.offsets.push(offset);
        self.current_index = self.offsets.len() - 1;
    }

    fn previous_one(&mut self) -> usize {
        self.current_index = self.current_index.saturating_sub(1);
        self.offsets[self.current_index]
    }

    fn next_one(&mut self) -> usize {
        self.current_index = std::cmp::min(self.current_index + 1, self.offsets.len() - 1);
        self.offsets[self.current_index]
    }
}

#[derive(Debug)]
pub struct Window {
    pub width: usize,
    pub height: usize,
    // offset of first line start instead of top-left corner of window
    offset: usize,
    pub horizontal_shift: usize,
    offset_history: OffsetHistory,
}

impl Window {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            width: width as usize,
            height: height as usize - 1,
            offset: 0,
            horizontal_shift: 0,
            offset_history: OffsetHistory::new(),
        })
    }

    pub fn move_offset_by(&mut self, distance: usize, direction: Direction) {
        assert!(direction.is_vertical());
        if direction == Direction::Up {
            self.set_offset(self.offset.saturating_sub(distance));
        } else {
            self.set_offset(self.offset + distance);
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height - 1;
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
        self.offset_history.push(offset);
    }

    pub fn goto_previous_offset(&mut self) {
        self.offset = self.offset_history.previous_one();
    }

    pub fn goto_next_offset(&mut self) {
        self.offset = self.offset_history.next_one();
    }
}
