use std::{collections::BTreeSet, ops::Range};

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Stylize},
};

use crate::render::{render_line, RenderScheme, Renderer};

#[derive(Debug, PartialEq)]
enum HighlightFlag {
    On,
    Off,
}

impl HighlightFlag {
    fn toggle(&mut self) {
        match self {
            Self::On => *self = Self::Off,
            Self::Off => *self = Self::On,
        }
    }
}

#[derive(Debug)]
struct HighlightOption {
    foreground_color: Color,
    background_color: Color,
}

impl HighlightOption {
    fn new(foreground_color: Color, background_color: Color) -> Self {
        Self {
            foreground_color,
            background_color,
        }
    }

    fn from_slot_index(slot_index: usize) -> Self {
        match slot_index {
            1 => Self::new(Color::Black, Color::Grey),
            2 => Self::new(Color::Black, Color::Blue),
            3 => Self::new(Color::Black, Color::Cyan),
            4 => Self::new(Color::Black, Color::Green),
            5 => Self::new(Color::Black, Color::Yellow),
            6 => Self::new(Color::Magenta, Color::Reset),
            7 => Self::new(Color::Blue, Color::Reset),
            8 => Self::new(Color::Cyan, Color::Reset),
            9 => Self::new(Color::Green, Color::Reset),
            0 => Self::new(Color::Yellow, Color::Reset),
            _ => unreachable!(),
        }
    }

    fn render_schemas(&self) -> Vec<RenderScheme> {
        vec![
            RenderScheme::ForegroundColor(self.foreground_color),
            RenderScheme::BackgroundColor(self.background_color),
        ]
    }
}

#[derive(Debug, PartialEq)]
enum AdvancedAction {
    Nothing,
    Fold,
    Exclusive,
}

impl AdvancedAction {
    fn toggle_fold(&mut self) {
        match self {
            Self::Fold => *self = Self::Nothing,
            _ => *self = Self::Fold,
        }
    }

    fn toggle_exclusive(&mut self) {
        match self {
            Self::Exclusive => *self = Self::Nothing,
            _ => *self = Self::Exclusive,
        }
    }
}

#[derive(Debug, PartialEq)]
enum PatternType {
    Raw,
    Regex,
}

impl PatternType {
    fn toggle(&mut self) {
        match self {
            Self::Raw => *self = Self::Regex,
            Self::Regex => *self = Self::Raw,
        }
    }
}

fn array_index_to_slot_index(index: usize) -> usize {
    assert!(index <= 9);
    (index + 1) % 10
}

fn array_index_from_slot_index(slot_index: usize) -> usize {
    assert!(slot_index <= 9);
    (slot_index + 9) % 10
}

#[derive(Debug)]
struct FinderSlot {
    slot_index: usize,
    highlight_flag: HighlightFlag,
    highlight_option: HighlightOption,
    advanced_action: AdvancedAction,
    pattern_type: PatternType,
    pattern: Option<String>,
}

impl FinderSlot {
    fn from_slot_array_index(index: usize) -> Self {
        let slot_index = array_index_to_slot_index(index);
        Self {
            slot_index,
            highlight_flag: HighlightFlag::On,
            highlight_option: HighlightOption::from_slot_index(slot_index),
            advanced_action: AdvancedAction::Nothing,
            pattern_type: PatternType::Raw,
            pattern: None,
        }
    }

    fn reset(&mut self) {
        self.highlight_flag = HighlightFlag::On;
        self.advanced_action = AdvancedAction::Nothing;
        self.pattern_type = PatternType::Raw;
        self.pattern = None;
    }
}

const FINDER_SLOT_COUNT: usize = 10;

#[derive(Debug)]
pub struct Finder {
    slots: [FinderSlot; FINDER_SLOT_COUNT],
    active_slots: BTreeSet<usize>,
    menu_active: bool,
}

impl Finder {
    pub fn new() -> Self {
        Self {
            slots: core::array::from_fn(FinderSlot::from_slot_array_index),
            active_slots: BTreeSet::from_iter([1]),
            menu_active: false,
        }
    }

    pub fn is_menu_active(&self) -> bool {
        self.menu_active
    }

    pub fn update_search_pattern(&mut self, pattern: &str) {
        assert!(self.active_slots.len() == 1);
        let index = array_index_from_slot_index(*self.active_slots.iter().next().unwrap());
        self.slots[index].pattern = Some(pattern.to_string());
    }

    pub fn active_search_patterns(&self) -> Vec<&str> {
        let mut active_patterns = vec![];
        for slot_index in self.active_slots.iter() {
            let index = array_index_from_slot_index(*slot_index);
            if let Some(pattern) = &self.slots[index].pattern {
                active_patterns.push(pattern.as_str());
            }
        }
        active_patterns
    }

    pub fn handle_event(&mut self, action: FinderAction) {
        match action {
            FinderAction::MenuOn => self.menu_active = true,
            FinderAction::MenuOff => self.menu_active = false,
            FinderAction::AddActiveSlotStart => unreachable!(),
            FinderAction::RemoveActiveSlotStart => unreachable!(),
            FinderAction::SwitchActiveSlot(index) => self.set_active_slot(index),
            FinderAction::AddActiveSlot(index) => self.add_active_slot(index),
            FinderAction::RemoveActiveSlot(index) => self.remove_active_slot(index),
            FinderAction::AddOrRemoveActiveSlotCancel => {}
            FinderAction::ToggleHighlightFlag => self.toggle_highlight_flag(),
            FinderAction::ToggleFoldAction => self.toggle_fold_action(),
            FinderAction::ToggleExclusiveAction => self.toggle_exclusive_action(),
            FinderAction::TogglePatternType => self.toggle_pattern_type(),
            FinderAction::ResetSlot => self.reset_active_slots(),
        }
    }

    pub fn active_slots(&self) -> &BTreeSet<usize> {
        &self.active_slots
    }

    pub fn set_active_slot(&mut self, slot_index: usize) {
        self.active_slots = BTreeSet::from_iter([slot_index]);
    }

    pub fn add_active_slot(&mut self, slot_index: usize) {
        self.active_slots.insert(slot_index);
    }

    pub fn remove_active_slot(&mut self, slot_index: usize) {
        if self.active_slots.len() > 1 {
            self.active_slots.remove(&slot_index);
            assert!(!self.active_slots.is_empty());
        }
    }

    pub fn toggle_highlight_flag(&mut self) {
        for index in self.active_slots.iter() {
            self.slots[array_index_from_slot_index(*index)]
                .highlight_flag
                .toggle();
        }
    }

    pub fn toggle_fold_action(&mut self) {
        for index in self.active_slots.iter() {
            let slot = &mut self.slots[array_index_from_slot_index(*index)];
            slot.advanced_action.toggle_fold();
        }
    }

    pub fn toggle_exclusive_action(&mut self) {
        for index in self.active_slots.iter() {
            let slot = &mut self.slots[array_index_from_slot_index(*index)];
            slot.advanced_action.toggle_exclusive();
        }
    }

    pub fn toggle_pattern_type(&mut self) {
        for index in self.active_slots.iter() {
            self.slots[array_index_from_slot_index(*index)]
                .pattern_type
                .toggle();
        }
    }

    pub fn reset_active_slots(&mut self) {
        for index in self.active_slots.iter() {
            self.slots[array_index_from_slot_index(*index)].reset();
        }
    }

    pub fn can_pass_advance_action(&self, line: &str) -> bool {
        let fold_patterns = self
            .slots
            .iter()
            .filter_map(|s| {
                if s.advanced_action == AdvancedAction::Fold {
                    s.pattern.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if fold_patterns.iter().any(|p| line.contains(p)) {
            return false;
        }

        let exclusive_patterns = self
            .slots
            .iter()
            .filter_map(|s| {
                if s.advanced_action == AdvancedAction::Exclusive {
                    s.pattern.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if !exclusive_patterns.is_empty() && exclusive_patterns.iter().all(|ep| !line.contains(ep))
        {
            return false;
        }

        true
    }

    pub fn render_body_area(&self, renderer: &mut Renderer) {
        let body_area_height = renderer.buffer.len() - renderer.popup_menu_render_text.len();
        renderer.buffer = renderer
            .buffer
            .iter()
            .take(body_area_height)
            .map(|line| self.render_line(line))
            .collect()
    }

    fn render_line(&self, line: &str) -> String {
        let mut range_scheme: Vec<(Range<usize>, Vec<RenderScheme>)> = vec![];
        // active slots have higher priority than inactive ones
        let (active, inactive): (Vec<_>, Vec<_>) = self
            .slots
            .iter()
            .partition(|slot| self.active_slots.contains(&slot.slot_index));
        for slot in active.iter().chain(inactive.iter()) {
            if slot.highlight_flag == HighlightFlag::Off {
                continue;
            }
            // todo: handle regex pattern type
            // todo: find all appearance instead of only the first one
            // todo: handle wrapped match
            if let Some(pattern) = &slot.pattern {
                if let Some(start) = line.find(pattern) {
                    let end = start + pattern.len();
                    if range_scheme
                        .iter()
                        .all(|(range, _)| !(range.start < end && start < range.end))
                    {
                        range_scheme.push((start..end, slot.highlight_option.render_schemas()));
                    }
                }
            }
        }
        render_line(line, range_scheme)
    }

    pub fn render_status_bar(&self, renderer: &mut Renderer, space_count: usize) {
        if space_count < 40 {
            return;
        }
        let text = &mut renderer.status_bar_render_text;
        let end = text.len() - 5;
        let mut slot_str = String::default();
        for slot in self.slots.iter() {
            let maybe_cursor = if self.active_slots.contains(&slot.slot_index) {
                '*'
            } else {
                ' '
            };
            let rendered_index = if slot.pattern.is_some() {
                slot.slot_index
                    .to_string()
                    .with(slot.highlight_option.foreground_color)
                    .on(slot.highlight_option.background_color)
                    .to_string()
            } else {
                slot.slot_index.to_string().dim().to_string()
            };
            slot_str.push_str(&format!(" {maybe_cursor}{}", rendered_index));
        }
        slot_str.push_str(" |");
        text.replace_range(end - 32..end, &slot_str);
    }

    pub fn render_menu(&self, renderer: &mut Renderer, window_width: usize, window_height: usize) {
        const MENU_HEIGHT: usize = 11;
        const MENU_MIN_WIDTH: usize = 50;
        const FINDER_MENU_STR: &str = " Finder Menu ";
        let width = std::cmp::max(window_width, 20);
        let mut title = "=".repeat(width);
        let begin = (width - FINDER_MENU_STR.len()) / 2;
        title.replace_range(begin..begin + FINDER_MENU_STR.len(), FINDER_MENU_STR);
        title.truncate(window_width);
        if window_height < MENU_HEIGHT + 5 || window_width < MENU_MIN_WIDTH {
            renderer.status_bar_render_text = title;
            return;
        }

        let menu_canvas = &mut renderer.popup_menu_render_text;
        menu_canvas.clear();
        menu_canvas.push(title);
        for slot in self.slots.iter() {
            let maybe_cursor = if self.active_slots.contains(&slot.slot_index) {
                '*'
            } else {
                ' '
            };
            let mut line = format!(
                " {maybe_cursor} {} | On Off | Fold Exclusive | Raw Regex | {}",
                slot.slot_index,
                slot.pattern.as_ref().unwrap_or(&String::default())
            );
            line.truncate(window_width);

            let mut range_scheme = vec![];
            range_scheme.push((
                3..4,
                vec![
                    RenderScheme::ForegroundColor(slot.highlight_option.foreground_color),
                    RenderScheme::BackgroundColor(slot.highlight_option.background_color),
                ],
            ));
            if slot.highlight_flag != HighlightFlag::On {
                range_scheme.push((7..9, vec![RenderScheme::Dim]));
            }
            if slot.highlight_flag != HighlightFlag::Off {
                range_scheme.push((10..13, vec![RenderScheme::Dim]));
            }
            if slot.advanced_action != AdvancedAction::Fold {
                range_scheme.push((16..20, vec![RenderScheme::Dim]));
            }
            if slot.advanced_action != AdvancedAction::Exclusive {
                range_scheme.push((21..30, vec![RenderScheme::Dim]));
            }
            if slot.pattern_type != PatternType::Raw {
                range_scheme.push((33..36, vec![RenderScheme::Dim]));
            }
            if slot.pattern_type != PatternType::Regex {
                range_scheme.push((37..42, vec![RenderScheme::Dim]));
            }
            let rendered_line = render_line(&line, range_scheme);
            menu_canvas.push(rendered_line);
        }
        assert!(menu_canvas.len() == MENU_HEIGHT);
        renderer.status_bar_render_text = String::default();
    }
}

#[derive(Debug, PartialEq)]
pub enum FinderAction {
    SwitchActiveSlot(usize),
    AddActiveSlotStart,
    AddActiveSlot(usize),
    RemoveActiveSlotStart,
    RemoveActiveSlot(usize),
    AddOrRemoveActiveSlotCancel,
    ToggleHighlightFlag,
    ToggleFoldAction,
    ToggleExclusiveAction,
    TogglePatternType,
    ResetSlot,
    MenuOn,
    MenuOff,
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
enum FinderEventParserState {
    #[default]
    Normal,
    ParsedAdd,
    ParsedRemove,
}

#[derive(Debug, Default)]
pub struct FinderEventParser {
    state: FinderEventParserState,
    menu_active: bool,
}

impl FinderEventParser {
    pub fn try_parse_raw_event(&mut self, key: &KeyEvent) -> Option<FinderAction> {
        if key.modifiers != KeyModifiers::NONE && key.modifiers != KeyModifiers::SHIFT {
            return None;
        }
        match key.code {
            KeyCode::Char('+') => {
                if self.state == FinderEventParserState::Normal {
                    self.state = FinderEventParserState::ParsedAdd;
                    return Some(FinderAction::AddActiveSlotStart);
                }
            }
            KeyCode::Char('-') => {
                if self.state == FinderEventParserState::Normal {
                    self.state = FinderEventParserState::ParsedRemove;
                    return Some(FinderAction::RemoveActiveSlotStart);
                }
            }
            KeyCode::Char(index @ '0'..='9') => {
                let index = index as usize - '0' as usize;
                let state = self.state;
                self.state = FinderEventParserState::Normal;
                return match state {
                    FinderEventParserState::Normal => Some(FinderAction::SwitchActiveSlot(index)),
                    FinderEventParserState::ParsedAdd => Some(FinderAction::AddActiveSlot(index)),
                    FinderEventParserState::ParsedRemove => {
                        Some(FinderAction::RemoveActiveSlot(index))
                    }
                };
            }
            KeyCode::Esc => {
                if self.state != FinderEventParserState::Normal {
                    self.state = FinderEventParserState::Normal;
                    return Some(FinderAction::AddOrRemoveActiveSlotCancel);
                }
                if self.menu_active {
                    self.menu_active = false;
                    return Some(FinderAction::MenuOff);
                }
            }
            KeyCode::Char('o') => {
                if self.state == FinderEventParserState::Normal {
                    return Some(FinderAction::ToggleHighlightFlag);
                }
            }
            KeyCode::Char('f') => {
                if self.state == FinderEventParserState::Normal {
                    return Some(FinderAction::ToggleFoldAction);
                }
            }
            KeyCode::Char('e') => {
                if self.state == FinderEventParserState::Normal {
                    return Some(FinderAction::ToggleExclusiveAction);
                }
            }
            KeyCode::Char('r') => {
                if self.state == FinderEventParserState::Normal {
                    return Some(FinderAction::TogglePatternType);
                }
            }
            KeyCode::Char('x') => {
                if self.state == FinderEventParserState::Normal {
                    return Some(FinderAction::ResetSlot);
                }
            }
            KeyCode::Char('m') => {
                if self.state == FinderEventParserState::Normal {
                    return if self.menu_active {
                        self.menu_active = false;
                        Some(FinderAction::MenuOff)
                    } else {
                        self.menu_active = true;
                        Some(FinderAction::MenuOn)
                    };
                }
            }
            _ => return None,
        }
        None
    }
}
