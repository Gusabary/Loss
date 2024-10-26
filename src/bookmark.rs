use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{canvas::Canvas, event_source::Direction, render::LineWithRenderScheme};

pub const BOOKMARK_NAME_MAX_LEN: usize = 50;

#[derive(Debug, Default)]
pub struct BookmarkStore {
    bookmarks: BTreeMap<String, (usize, String)>,
    menu_index: Option<usize>,
    filtered_bookmarks: Vec<(String, usize, String)>,
    filter_content: String,
}

impl BookmarkStore {
    pub fn new_bookmark(&mut self, name: &str, offset: usize, line: &str) {
        self.bookmarks
            .insert(name.to_string(), (offset, line.to_string()));
    }

    pub fn is_active(&self) -> bool {
        self.menu_index.is_some()
    }

    pub fn handle_enter_event(&mut self) -> Option<&(String, usize, String)> {
        if self.filtered_bookmarks.is_empty() {
            None
        } else {
            assert!(self.menu_index.unwrap() < self.filtered_bookmarks.len());
            let bookmark = &self.filtered_bookmarks[self.menu_index.unwrap()];
            self.menu_index = None;
            Some(bookmark)
        }
    }

    pub fn handle_other_event(&mut self, action: BookmarkMenuAction) {
        match action {
            BookmarkMenuAction::Start => {
                assert!(self.menu_index.is_none());
                self.menu_index = Some(0);
                self.load_filtered_bookmarks("");
            }
            BookmarkMenuAction::Arrow(direction) => {
                if self.filtered_bookmarks.is_empty() {
                    return;
                }
                assert!(direction.is_vertical());
                self.menu_index = Some(
                    if direction == Direction::Up {
                        self.menu_index.unwrap() + self.filtered_bookmarks.len() - 1
                    } else {
                        self.menu_index.unwrap() + 1
                    } % self.filtered_bookmarks.len(),
                );
            }
            BookmarkMenuAction::Content(filter_content) => {
                let prev_bookmark = if self.filtered_bookmarks.is_empty() {
                    String::default()
                } else {
                    self.current_bookmark().to_string()
                };
                self.load_filtered_bookmarks(&filter_content);
                if let Some(index) = self
                    .filtered_bookmarks
                    .iter()
                    .position(|(name, _, _)| *name == prev_bookmark)
                {
                    self.menu_index = Some(index);
                } else {
                    // if the selected bookmark disappears after filter change, reset the cursor to 0
                    self.menu_index = Some(0);
                }
                self.filter_content = filter_content;
            }
            BookmarkMenuAction::Enter => unreachable!(),
            BookmarkMenuAction::Cancel => {
                self.menu_index = None;
            }
        }
    }

    fn current_bookmark(&self) -> &str {
        assert!(self.menu_index.unwrap() < self.filtered_bookmarks.len());
        &self.filtered_bookmarks[self.menu_index.unwrap()].0
    }

    fn load_filtered_bookmarks(&mut self, filter_content: &str) {
        self.filtered_bookmarks = self
            .bookmarks
            .iter()
            .filter(|(name, _)| name.contains(filter_content))
            .map(|(name, (offset, line))| (name.clone(), *offset, line.clone()))
            .collect();
    }

    pub fn render(&self, canvas: &mut Canvas, window_width: usize, window_height: usize) {
        const MENU_HEIGHT: usize = 10;
        const BOOK_MENU_STR: &str = " Bookmark Menu ";
        let width = std::cmp::max(window_width, 20);
        let mut title = "=".repeat(width);
        let begin = (width - BOOK_MENU_STR.len()) / 2;
        title.replace_range(begin..begin + BOOK_MENU_STR.len(), BOOK_MENU_STR);
        title.truncate(window_width);
        if window_height < MENU_HEIGHT + 5 {
            canvas.status_bar = LineWithRenderScheme::new(&title);
            canvas.cursor_pos_x = None;
            return;
        }
        canvas.popup_menu.clear();
        canvas.popup_menu.push(LineWithRenderScheme::new(&title));
        let menu_index = self.menu_index.unwrap();
        let displayed_bookmarkes: Vec<_> =
            if menu_index + MENU_HEIGHT > self.filtered_bookmarks.len() {
                self.filtered_bookmarks
                    .iter()
                    .enumerate()
                    .rev()
                    .take(MENU_HEIGHT - 1)
                    .rev()
                    .collect()
            } else {
                self.filtered_bookmarks
                    .iter()
                    .enumerate()
                    .skip(menu_index)
                    .take(MENU_HEIGHT - 1)
                    .collect()
            };
        for (index, (name, _, line)) in displayed_bookmarkes.iter() {
            let maybe_cursor = if *index == menu_index { '>' } else { ' ' };
            let raw_line = &format!(" {maybe_cursor} {name:<BOOKMARK_NAME_MAX_LEN$}    {line}");
            let menu_line = LineWithRenderScheme::new(raw_line).truncate(window_width);
            canvas.popup_menu.push(menu_line);
        }
        assert!(canvas.popup_menu.len() <= MENU_HEIGHT);
        canvas
            .popup_menu
            .resize(MENU_HEIGHT, LineWithRenderScheme::default());

        let status_bar_text = &format!("Filter bookmark: {}", self.filter_content);
        canvas.status_bar = LineWithRenderScheme::new(status_bar_text).truncate(window_width);
        canvas.cursor_pos_x = Some(status_bar_text.len());
    }
}

#[derive(Debug, PartialEq)]
pub enum BookmarkMenuAction {
    Start,
    Arrow(Direction),
    Content(String),
    Enter,
    Cancel,
}

#[derive(Debug, Default)]
pub struct BookMarkMenu {
    active: bool,
    filter_content: String,
}

impl BookMarkMenu {
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) {
        assert!(!self.active);
        self.active = true;
    }

    pub fn handle_raw_event(&mut self, key: &KeyEvent) -> Option<BookmarkMenuAction> {
        assert!(self.active);
        if key.modifiers != KeyModifiers::NONE && key.modifiers != KeyModifiers::SHIFT {
            None
        } else {
            match key.code {
                KeyCode::Char(c) => {
                    self.filter_content.push(c);
                    Some(BookmarkMenuAction::Content(self.filter_content.to_string()))
                }
                KeyCode::Backspace => {
                    self.filter_content.pop();
                    Some(BookmarkMenuAction::Content(self.filter_content.to_string()))
                }
                KeyCode::Enter => {
                    self.active = false;
                    Some(BookmarkMenuAction::Enter)
                }
                KeyCode::Esc => {
                    self.active = false;
                    Some(BookmarkMenuAction::Cancel)
                }
                KeyCode::Up => Some(BookmarkMenuAction::Arrow(Direction::Up)),
                KeyCode::Down => Some(BookmarkMenuAction::Arrow(Direction::Down)),
                _ => None,
            }
        }
    }
}
