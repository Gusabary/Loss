use std::fs::File;

use crate::{
    document::Document,
    event_source::{Direction, Event, EventSource},
    render::{RenderOptions, Renderer},
    window::Window,
};
use anyhow::{Ok, Result};
use crossterm::terminal;
use log::info;

pub struct Manager {
    document: Document<File>,
    window: Window,
    event_source: EventSource,
    renderer: Renderer,
}

impl Manager {
    pub fn new(filename: &str) -> Result<Manager> {
        info!("[new] ===== manager created: {filename} =====");
        Ok(Manager {
            document: Document::<File>::open_file(filename)?,
            window: Window::new()?,
            event_source: EventSource {},
            renderer: Renderer::default(),
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
        let lines = self
            .document
            .query_lines(self.window.offset, self.window.height)?;

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
                self.window.offset = self.window.offset.saturating_add(distance);
            }
            Direction::Left => panic!("not supported yet"),
            Direction::Right => panic!("not supported yet"),
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
