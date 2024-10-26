use std::fs::File;

use crate::{
    bookmark::{BookmarkMenuAction, BookmarkStore, BOOKMARK_NAME_MAX_LEN},
    canvas::{clear_screen_and_reset_cursor, Canvas},
    document::Document,
    event_source::{Direction, Event, EventSource},
    finder::{Finder, FinderAction},
    log_timestamp::parse_log_timestamp,
    prompt::PromptAction,
    render::LineWithRenderScheme,
    status_bar::StatusBar,
    window::Window,
};
use anyhow::{Ok, Result};
use log::info;

#[derive(Debug, Default)]
struct Context {
    raw_lines: Vec<String>,
    searching_direction: Option<Direction>,
    jumping_direction: Option<Direction>,
    wrap_lines: bool,
    need_rerender: bool,
}

#[derive(Debug, PartialEq)]
enum Mode {
    Normal,
    Follow,
}

pub struct Manager {
    document: Document<File>,
    window: Window,
    status_bar: StatusBar,
    event_source: EventSource,
    bookmark_store: BookmarkStore,
    finder: Finder,
    context: Context,
    canvas: Canvas,
    mode: Mode,
}

impl Manager {
    pub fn new(filename: &str) -> Result<Manager> {
        info!("[new] ===== manager created: {filename} =====");
        Ok(Manager {
            document: Document::<File>::open_file(filename)?,
            window: Window::new()?,
            status_bar: StatusBar::default(),
            event_source: EventSource::default(),
            bookmark_store: BookmarkStore::default(),
            finder: Finder::new(),
            context: Context::default(),
            canvas: Canvas::default(),
            mode: Mode::Normal,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.context.need_rerender = true;
        loop {
            self.fill_canvas_and_render()?;
            let should_exit = self.listen_and_dispatch_event()?;
            self.ensure_consistency()?;
            if should_exit {
                clear_screen_and_reset_cursor()?;
                return Ok(());
            }
        }
    }

    fn fill_canvas_and_render(&mut self) -> Result<()> {
        if !self.context.need_rerender {
            self.context.need_rerender = true;
            return Ok(());
        }
        self.context.raw_lines = self
            .document
            .query_lines(self.window.offset(), self.window.height)?;

        self.canvas.clear();
        for line in self.context.raw_lines.iter() {
            if !self.finder.can_pass_advance_action(line) {
                continue;
            }
            let line_with_render_scheme = self.finder.attach_render_scheme(line);
            if self.context.wrap_lines {
                for idx in 0..=line.len() / self.window.width {
                    let start = idx * self.window.width;
                    let end = std::cmp::min((idx + 1) * self.window.width, line.len());
                    let substr = line_with_render_scheme.substr(start..end);
                    self.canvas.body_area.push(substr);
                }
            } else {
                let start = self.window.horizontal_shift;
                let end = start + self.window.width;
                let substr = line_with_render_scheme.substr(start..end);
                self.canvas.body_area.push(substr);
            }
        }
        self.canvas
            .body_area
            .resize(self.window.height, LineWithRenderScheme::new("~"));

        if self.bookmark_store.is_active() {
            self.bookmark_store
                .render(&mut self.canvas, self.window.width, self.window.height);
        } else if self.finder.is_menu_active() {
            self.finder
                .render_menu(&mut self.canvas, self.window.width, self.window.height);
        } else {
            let ratio = self.document.percent_ratio_of_offset(self.window.offset());
            self.status_bar.set_ratio(ratio);
            if let Some(space_count) = self.status_bar.render(&mut self.canvas, self.window.width) {
                self.finder.render_status_bar(&mut self.canvas, space_count);
            }
        }
        self.canvas.render()?;
        Ok(())
    }

    fn listen_and_dispatch_event(&mut self) -> Result<bool> {
        if self.mode != Mode::Normal {
            if self.event_source.check_for_interrupt()? {
                self.mode = Mode::Normal;
                self.status_bar.clear_text();
            } else if self.mode == Mode::Follow {
                self.seek_to_end()?;
            }
            return Ok(false);
        }
        let event = self.event_source.wait_for_event()?;
        info!("[run] new event: {:?}", event);
        match event {
            Event::Exit => return Ok(true),
            Event::ToggleWrapLine => self.context.wrap_lines = !self.context.wrap_lines,
            Event::WindowMove(direction, step) => self.on_window_move_event(direction, step)?,
            Event::Search(action) => self.on_search_event(action)?,
            Event::SearchNext => self.search_next(Direction::Down, true)?,
            Event::SearchPrevious => self.search_next(Direction::Up, true)?,
            Event::SeekToEnd => self.seek_to_end()?,
            Event::SeekToHome => self.window.set_offset(0),
            Event::JumpToTimestamp(action) => self.on_jump_to_timestamp_event(action)?,
            Event::JumpByLines(action) => self.on_jump_by_lines_event(action)?,
            Event::TerminalResize(width, height) => self.window.resize(width, height),
            Event::NewBookmark(action) => self.on_new_bookmark_event(action)?,
            Event::GotoBookmark(action) => self.on_bookmark_menu_event(action)?,
            Event::UndoWindowVerticalMove => self.window.goto_previous_offset(),
            Event::RedoWindowVerticalMove => self.window.goto_next_offset(),
            Event::FinderOperation(action) => self.on_finder_event(action)?,
            Event::Follow => self.enter_follow_mode()?,
        }
        Ok(false)
    }

    fn on_window_move_event(&mut self, direction: Direction, step: usize) -> Result<()> {
        match direction {
            Direction::Up => {
                let distance = self
                    .document
                    .query_distance_to_above_n_lines(self.window.offset(), step)?;
                self.window.move_offset_by(distance, direction);
            }
            Direction::Down => {
                let distance = self
                    .document
                    .query_distance_to_below_n_lines(self.window.offset(), step)?;
                self.window.move_offset_by(distance, direction);
            }
            Direction::Left => {
                if !self.context.wrap_lines {
                    self.window.horizontal_shift =
                        self.window.horizontal_shift.saturating_sub(step);
                }
            }
            Direction::Right => {
                if !self.context.wrap_lines {
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
                assert!(direction.unwrap().is_vertical());
                self.context.searching_direction = direction;
                self.status_bar.set_text("Search: ");
            }
            PromptAction::Content(content) => {
                self.status_bar.set_text(&format!("Search: {content}"));
            }
            PromptAction::Cancel => {
                self.context.searching_direction = None;
                self.status_bar.clear_text();
            }
            PromptAction::Enter(content) => {
                if self.finder.active_slots().len() > 1 {
                    // todo: advance this to /
                    self.status_bar.set_oneoff_error_text(
                        "Cannot search with more than one active Finder slot",
                    );
                } else {
                    self.status_bar.clear_text();
                    self.finder.update_search_pattern(&content);
                    self.search_next(self.context.searching_direction.unwrap(), false)?;
                    self.context.searching_direction = None;
                }
            }
        }
        Ok(())
    }

    fn search_next(&mut self, direction: Direction, from_next_event: bool) -> Result<()> {
        assert!(direction.is_vertical());
        let search_predict = |line: &str| self.finder.can_satisfy_active_search_patterns(line);
        let mut extra_distance = 0;
        let distance = if direction == Direction::Up {
            self.document
                .query_distance_to_prev_match(self.window.offset(), search_predict)?
        } else {
            if from_next_event {
                extra_distance = self
                    .document
                    .query_distance_to_below_n_lines(self.window.offset(), 1)?;
            }
            self.document.query_distance_to_next_match(
                self.window.offset() + extra_distance,
                search_predict,
            )?
        };
        if let Some(distance) = distance {
            self.window
                .move_offset_by(distance + extra_distance, direction);
        } else {
            self.status_bar.set_oneoff_error_text("Not found");
        }
        Ok(())
    }

    fn seek_to_end(&mut self) -> Result<()> {
        let document_updated = self.document.update_docsize_and_lastline()?;
        self.context.need_rerender = document_updated;
        let distance = self.document.query_distance_to_above_n_lines(
            self.document.last_line_start_offset(),
            self.window.height.saturating_sub(1),
        )?;
        self.window.set_offset(
            self.document
                .last_line_start_offset()
                .saturating_sub(distance),
        );
        Ok(())
    }

    fn on_jump_to_timestamp_event(&mut self, action: PromptAction) -> Result<()> {
        match action {
            PromptAction::Start(direction) => {
                assert!(direction.is_none());
                self.status_bar.set_text("Jump to timestamp");
            }
            PromptAction::Content(content) => {
                self.status_bar
                    .set_text(&format!("Jump to timestamp: {content}"));
            }
            PromptAction::Cancel => {
                self.status_bar.clear_text();
            }
            PromptAction::Enter(content) => {
                self.status_bar.clear_text();
                let (date, time) = parse_log_timestamp(&content);
                if let Some(time) = time {
                    if let Some(offset) = self.document.query_offset_by_timestamp(date, time)? {
                        self.window.set_offset(offset)
                    } else {
                        self.status_bar
                            .set_oneoff_error_text("Cannot jump to timestamp");
                    }
                } else {
                    self.status_bar.set_oneoff_error_text("Invalid timestamp");
                }
            }
        }
        Ok(())
    }

    fn on_jump_by_lines_event(&mut self, action: PromptAction) -> Result<()> {
        match action {
            PromptAction::Start(direction) => {
                assert!(direction.unwrap().is_vertical());
                self.context.jumping_direction = direction;
                let s = direction.as_ref().unwrap().above_or_below();
                self.status_bar.set_text(&format!("Jump to {s} N lines: "));
            }
            PromptAction::Content(content) => {
                let s = self
                    .context
                    .jumping_direction
                    .as_ref()
                    .unwrap()
                    .above_or_below();
                self.status_bar
                    .set_text(&format!("Jump to {s} N lines: {content}"));
            }
            PromptAction::Cancel => {
                self.context.jumping_direction = None;
                self.status_bar.clear_text();
            }
            PromptAction::Enter(content) => {
                self.status_bar.clear_text();
                if let std::result::Result::Ok(step) = content.parse::<usize>() {
                    self.on_window_move_event(self.context.jumping_direction.unwrap(), step)?;
                } else {
                    self.status_bar.set_oneoff_error_text("Invalid line count");
                }
                self.context.jumping_direction = None;
            }
        }
        Ok(())
    }

    fn on_new_bookmark_event(&mut self, action: PromptAction) -> Result<()> {
        match action {
            PromptAction::Start(direction) => {
                assert!(direction.is_none());
                self.status_bar.set_text("New bookmark: ");
            }
            PromptAction::Content(content) => {
                self.status_bar
                    .set_text(&format!("New bookmark: {content}"));
            }
            PromptAction::Cancel => {
                self.status_bar.clear_text();
            }
            PromptAction::Enter(content) => {
                self.status_bar.clear_text();
                if content.len() > BOOKMARK_NAME_MAX_LEN {
                    self.status_bar.set_oneoff_error_text(&format!(
                        "Bookmark name should have no more than {BOOKMARK_NAME_MAX_LEN} chars"
                    ));
                } else {
                    let line = &self.document.query_lines(self.window.offset(), 1)?[0];
                    self.bookmark_store
                        .new_bookmark(&content, self.window.offset(), line);
                    self.status_bar
                        .set_oneoff_error_text(&format!("Bookmark saved: {content}"));
                }
            }
        }
        Ok(())
    }

    fn on_bookmark_menu_event(&mut self, action: BookmarkMenuAction) -> Result<()> {
        if action == BookmarkMenuAction::Enter {
            if let Some((bookmark_name, offset, _)) = self.bookmark_store.handle_enter_event() {
                self.window.set_offset(*offset);
                self.status_bar
                    .set_oneoff_error_text(&format!("Jumped to bookmark: {bookmark_name}"));
            }
        } else {
            self.bookmark_store.handle_other_event(action);
        }
        Ok(())
    }

    fn on_finder_event(&mut self, action: FinderAction) -> Result<()> {
        if action == FinderAction::AddActiveSlotStart {
            self.status_bar.set_text("Adding Finder active slot ...");
        } else if action == FinderAction::RemoveActiveSlotStart {
            self.status_bar.set_text("Removing Finder active slot ...");
        } else {
            self.status_bar.clear_text();
            self.finder.handle_event(action);
        }
        Ok(())
    }

    fn enter_follow_mode(&mut self) -> Result<()> {
        assert_eq!(self.mode, Mode::Normal);
        self.mode = Mode::Follow;
        self.status_bar
            .set_text("Waiting for data... (interrupt to abort)");
        Ok(())
    }

    fn ensure_consistency(&mut self) -> Result<()> {
        assert!(self.window.offset() <= self.document.last_line_start_offset());
        if self.window.offset() < self.document.last_line_start_offset() {
            self.document
                .assert_offset_is_at_line_start(self.window.offset())?;
        }
        Ok(())
    }
}
