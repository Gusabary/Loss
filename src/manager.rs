use std::fs::File;

use crate::{
    document::Document,
    event_source::{Direction, Event, EventSource},
    render::{RenderOptions, Renderer},
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
    event_source: EventSource,
    window_offset: usize,
    window_size: WindowSize,
    window_horizontal_offset: usize,
    renderer: Renderer,
}

impl Manager {
    pub fn new(filename: &str) -> Result<Manager> {
        info!("[new] ===== manager created: {filename} =====");
        Ok(Manager {
            document: Document::<File>::open_file(filename)?,
            event_source: EventSource {},
            window_offset: 0,
            window_size: WindowSize::from_terminal_size()?,
            window_horizontal_offset: 0,
            renderer: Renderer::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.fill_buffer_and_render()?;
            self.listen_and_dispatch_event()?;
            self.ensure_consistency()?;
        }
    }

    fn fill_buffer_and_render(&mut self) -> Result<()> {
        let lines = self
            .document
            .query_lines(self.window_offset, self.window_size.height)?;

        self.renderer.buffer.clear();
        for line in lines.iter() {
            if self.renderer.options.wrap_lines {
                if line.is_empty() {
                    self.renderer.buffer.push(String::default());
                    continue;
                }
                for wrapped_line in line
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(self.window_size.width)
                    .map(|chunk| chunk.iter().collect::<String>())
                {
                    self.renderer.buffer.push(wrapped_line);
                }
            } else {
                let displayed_line = if self.window_horizontal_offset >= line.len() {
                    ""
                } else {
                    let upper = std::cmp::min(
                        self.window_horizontal_offset + self.window_size.width,
                        line.len(),
                    );
                    &line[self.window_horizontal_offset..upper]
                };
                self.renderer.buffer.push(displayed_line.to_string());
            }
        }
        self.renderer.buffer.truncate(self.window_size.height);
        self.renderer.render()?;
        Ok(())
    }

    fn listen_and_dispatch_event(&mut self) -> Result<()> {
        let event = self.event_source.wait_for_event()?;
        info!("[run] new event: {:?}", event);
        match event {
            Event::Exit => return Ok(()),
            Event::ToggleWrapLine => {
                self.renderer.options.wrap_lines = !self.renderer.options.wrap_lines
            }
            Event::WindowMove(direction, step) => self.on_window_move_event(direction, step)?,
        }
        info!("[run] window_offset: {}", self.window_offset);
        Ok(())
    }

    fn on_window_move_event(&mut self, direction: Direction, step: usize) -> Result<()> {
        match direction {
            Direction::Up => {
                let distance = self
                    .document
                    .query_distance_to_above_n_lines(self.window_offset, step)?;
                self.window_offset = self.window_offset.saturating_sub(distance);
            }
            Direction::Down => {
                let distance = self
                    .document
                    .query_distance_to_below_n_lines(self.window_offset, step)?;
                self.window_offset = self.window_offset.saturating_add(distance);
            }
            Direction::Left => panic!("not supported yet"),
            Direction::Right => panic!("not supported yet"),
        }
        Ok(())
    }

    fn ensure_consistency(&mut self) -> Result<()> {
        assert!(self.window_offset <= self.document.last_line_start_offset());
        if self.window_offset < self.document.last_line_start_offset() {
            self.document
                .assert_offset_is_at_line_start(self.window_offset)?;
        }
        Ok(())
    }
}
