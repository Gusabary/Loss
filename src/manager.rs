use std::fs::File;

use crate::{
    document::Document,
    event_source::{Direction, Event, EventSource, PromptAction},
    render::{clear_screen_and_reset_cursor, Renderer},
    window::Window,
};
use anyhow::{Ok, Result};
use log::info;

#[derive(Debug, Default)]
struct Context {
    raw_lines: Vec<String>,
    searching_direction: Option<Direction>,
    searching_content: Option<String>,
}

pub struct Manager {
    document: Document<File>,
    window: Window,
    event_source: EventSource,
    renderer: Renderer,
    context: Context,
}

impl Manager {
    pub fn new(filename: &str) -> Result<Manager> {
        info!("[new] ===== manager created: {filename} =====");
        Ok(Manager {
            document: Document::<File>::open_file(filename)?,
            window: Window::new()?,
            event_source: EventSource::default(),
            renderer: Renderer::default(),
            context: Context::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.fill_buffer_and_render()?;
            let should_exit = self.listen_and_dispatch_event()?;
            self.ensure_consistency()?;
            if should_exit {
                clear_screen_and_reset_cursor()?;
                return Ok(());
            }
        }
    }

    fn fill_buffer_and_render(&mut self) -> Result<()> {
        self.context.raw_lines = self
            .document
            .query_lines(self.window.offset, self.window.height)?;

        self.renderer.buffer.clear();
        for line in self.context.raw_lines.iter() {
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
        self.renderer
            .buffer
            .resize(self.window.height, "~".to_string());
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
            Event::Search(action) => self.on_search_event(action)?,
            Event::Next => self.search_next(Direction::Down, true)?,
            Event::Previous => self.search_next(Direction::Up, true)?,
            Event::SeekToEnd => self.window.offset = self.document.last_line_start_offset(),
            Event::SeekToHome => self.window.offset = 0,
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
                self.window.move_offset_by(distance, direction);
            }
            Direction::Down => {
                let distance = self
                    .document
                    .query_distance_to_below_n_lines(self.window.offset, step)?;
                self.window.move_offset_by(distance, direction);
            }
            Direction::Left => {
                if !self.renderer.options.wrap_lines {
                    self.window.horizontal_shift =
                        self.window.horizontal_shift.saturating_sub(step);
                }
            }
            Direction::Right => {
                if !self.renderer.options.wrap_lines {
                    let max_line_len = self
                        .context
                        .raw_lines
                        .iter()
                        .map(|line| line.len())
                        .max()
                        .unwrap();
                    let max_window_shift = max_line_len.saturating_sub(self.window.width);
                    self.window.horizontal_shift =
                        std::cmp::min(self.window.horizontal_shift + step, max_window_shift);
                }
            }
        }
        Ok(())
    }

    fn on_search_event(&mut self, action: PromptAction) -> Result<()> {
        match action {
            PromptAction::Start(direction) => {
                assert!(direction.is_vertical());
                self.context.searching_direction = Some(direction);
                self.renderer.bottom_line_text = format!("Search: ");
            }
            PromptAction::Content(content) => {
                self.renderer.bottom_line_text = format!("Search: {content}")
            }
            PromptAction::Cancel => {
                self.context.searching_direction = None;
                self.renderer.bottom_line_text = String::default();
            }
            PromptAction::Enter(content) => {
                self.renderer.bottom_line_text = String::default();
                self.context.searching_content = Some(content.clone());
                self.search_next(self.context.searching_direction.unwrap(), false)?;
                self.context.searching_direction = None;
            }
        }
        Ok(())
    }

    fn search_next(&mut self, direction: Direction, from_next_event: bool) -> Result<()> {
        assert!(direction.is_vertical());
        if from_next_event && self.context.searching_content.is_none() {
            return Ok(());
        }
        let content = self.context.searching_content.as_ref().unwrap();
        let mut extra_distance = 0;
        let distance = if direction == Direction::Up {
            self.document
                .query_distance_to_prev_match(self.window.offset, content)?
        } else {
            if from_next_event {
                extra_distance = self
                    .document
                    .query_distance_to_below_n_lines(self.window.offset, 1)?;
            }
            self.document
                .query_distance_to_next_match(self.window.offset + extra_distance, content)?
        };
        if let Some(distance) = distance {
            self.window
                .move_offset_by(distance + extra_distance, direction);
            self.renderer.options.highlight_text = Some(content.clone());
        } else {
            self.renderer.oneoff_bottom_line_text = Some(format!("Not found"));
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
