use std::fs::File;

use crate::{
    document::Document,
    event_source::{Direction, Event, EventHub},
    render::{render, RenderOptions},
};
use anyhow::{Ok, Result};
use crossterm::terminal;
use log::info;

#[derive(Debug)]
pub struct WindowSize {
    height: usize,
    width: usize,
}

impl WindowSize {
    pub fn from_terminal_size() -> Result<WindowSize> {
        let (width, height) = terminal::size()?;
        // the last row is status bar
        Ok(WindowSize {
            height: height as usize - 1,
            width: width as usize,
        })
    }
}

pub struct Manager {
    document: Document<File>,
    event_hub: EventHub,
    window_offset: usize,
    window_size: WindowSize,
    render_options: RenderOptions,
}

impl Manager {
    pub fn new(filename: &str) -> Result<Manager> {
        info!("[new] manager created: {filename}");
        Ok(Manager {
            document: Document::<File>::open_file(filename)?,
            event_hub: EventHub {},
            window_offset: 0,
            window_size: WindowSize::from_terminal_size()?,
            render_options: RenderOptions {},
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let lines = self
                .document
                .query_lines(self.window_offset, self.window_size.height)?;

            render(&lines, &self.window_size, &self.render_options)?;

            let event = self.event_hub.wait_for_event()?;
            info!("[run] new event: {:?}", event);
            match event {
                Event::Exit => return Ok(()),
                Event::WindowMove(direction, step) => self.on_window_move_event(direction, step)?,
            }
            info!("[run] window_offset: {}", self.window_offset);
        }
    }

    fn on_window_move_event(&mut self, direction: Direction, step: usize) -> Result<()> {
        match direction {
            Direction::Up => panic!("not supported yet"),
            Direction::Down => {
                // todo: should use a first-class api of document
                let line_len = self.document.query_lines(self.window_offset, 1)?[0].len() + 1;
                self.window_offset += line_len;
            }
            Direction::Left => panic!("not supported yet"),
            Direction::Right => panic!("not supported yet"),
        }
        Ok(())
    }
}
