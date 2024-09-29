use std::fs::File;

use crate::{
    document::Document,
    event_hub::{Event, EventHub},
    render::{self, render, RenderOptions},
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
                Event::WindowMove(direction, step) => {}
            }
        }
    }
}
