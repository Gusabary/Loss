use std::fs::File;

use crate::{
    document::Document,
    event_source::{Direction, Event, EventSource},
    render::Renderer,
    window::Window,
};
use anyhow::{Ok, Result};
use log::info;

pub struct Manager {
    document: Document<File>,
    window: Window,
    event_source: EventSource,
    renderer: Renderer,
    raw_lines: Vec<String>,
}

impl Manager {
    pub fn new(filename: &str) -> Result<Manager> {
        info!("[new] ===== manager created: {filename} =====");
        Ok(Manager {
            document: Document::<File>::open_file(filename)?,
            window: Window::new()?,
            event_source: EventSource {},
            renderer: Renderer::default(),
            raw_lines: Vec::<String>::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.fill_buffer_and_render()?;
            let should_exit = self.listen_and_dispatch_event()?;
            self.ensure_consistency()?;
            if should_exit {
                return Ok(());
            }
        }
    }

    fn fill_buffer_and_render(&mut self) -> Result<()> {
        self.raw_lines = self
            .document
            .query_lines(self.window.offset, self.window.height)?;

        self.renderer.buffer.clear();
        for line in self.raw_lines.iter() {
            if self.renderer.options.wrap_lines {
                if line.is_empty() {
                    self.renderer.buffer.push(String::default());
                    continue;
                }
                for wrapped_line in line
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(self.window.width)
                    .map(|chunk| chunk.iter().collect::<String>())
                {
                    self.renderer.buffer.push(wrapped_line);
                }
            } else {
                let displayed_line = if self.window.horizontal_shift >= line.len() {
                    ""
                } else {
                    let upper =
                        std::cmp::min(self.window.horizontal_shift + self.window.width, line.len());
                    &line[self.window.horizontal_shift..upper]
                };
                self.renderer.buffer.push(displayed_line.to_string());
            }
        }
        self.renderer.buffer.truncate(self.window.height);
        self.renderer.render()?;
        Ok(())
    }

    fn listen_and_dispatch_event(&mut self) -> Result<bool> {
        let event = self.event_source.wait_for_event()?;
        info!("[run] new event: {:?}", event);
        match event {
            Event::Exit => return Ok(true),
            Event::ToggleWrapLine => {
                self.renderer.options.wrap_lines = !self.renderer.options.wrap_lines
            }
            Event::WindowMove(direction, step) => self.on_window_move_event(direction, step)?,
        }
        info!("[run] window.offset: {}", self.window.offset);
        Ok(false)
    }

    fn on_window_move_event(&mut self, direction: Direction, step: usize) -> Result<()> {
        match direction {
            Direction::Up => {
                let distance = self
                    .document
                    .query_distance_to_above_n_lines(self.window.offset, step)?;
                self.window.offset = self.window.offset.saturating_sub(distance);
            }
            Direction::Down => {
                let distance = self
                    .document
                    .query_distance_to_below_n_lines(self.window.offset, step)?;
                self.window.offset = self.window.offset + distance;
            }
            Direction::Left => {
                if !self.renderer.options.wrap_lines {
                    self.window.horizontal_shift =
                        self.window.horizontal_shift.saturating_sub(step);
                }
            }
            Direction::Right => {
                if !self.renderer.options.wrap_lines {
                    let max_line_len = self.raw_lines.iter().map(|line| line.len()).max().unwrap();
                    let max_window_shift = max_line_len.saturating_sub(self.window.width);
                    self.window.horizontal_shift =
                        std::cmp::min(self.window.horizontal_shift + step, max_window_shift);
                }
            }
        }
        Ok(())
    }

    fn ensure_consistency(&mut self) -> Result<()> {
        assert!(self.window.offset <= self.document.last_line_start_offset());
        if self.window.offset < self.document.last_line_start_offset() {
            self.document
                .assert_offset_is_at_line_start(self.window.offset)?;
        }
        Ok(())
    }
}
