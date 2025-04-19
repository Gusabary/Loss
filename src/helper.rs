use crate::{canvas::Canvas, render::LineWithRenderScheme};

#[derive(Default)]
pub struct HelperMenu {
    active: bool,
}

impl HelperMenu {
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn toggle_active(&mut self) {
        self.active = !self.active;
    }

    pub fn render(&mut self, canvas: &mut Canvas, window_width: usize, window_height: usize) {
        const MENU_HEIGHT: usize = 20;
        const MENU_MIN_WIDTH: usize = 75;
        const HELPER_MENU_STR: &str = " Helper Menu ";
        let width = std::cmp::max(window_width, 20);
        let mut title = "=".repeat(width);
        let begin = (width - HELPER_MENU_STR.len()) / 2;
        title.replace_range(begin..begin + HELPER_MENU_STR.len(), HELPER_MENU_STR);
        title.truncate(window_width);
        if window_height < MENU_HEIGHT + 5 || window_width < MENU_MIN_WIDTH {
            canvas.status_bar = LineWithRenderScheme::new(&title);
            canvas.cursor_pos_x = None;
            return;
        }
        populate_helper_menu(canvas, &title);
        canvas.status_bar = LineWithRenderScheme::default();
        canvas.cursor_pos_x = Some(0);
    }
}

#[rustfmt::skip]
fn populate_helper_menu(canvas: &mut Canvas, title: &str) {
    canvas.popup_menu.clear();
    canvas.popup_menu.push(LineWithRenderScheme::new(&title));
    canvas.popup_menu.push(LineWithRenderScheme::new("+------- basic commands -------+     +------- finder commands -------+"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| q: exit                      |     | +:   add active slot          |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| w: toggle wrap line          |     | -:   remove active slot       |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| /: search down               |     | 0-9: switch active slot       |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| ?: search up                 |     | o:   toggle highlight flag    |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| n: search next               |     | r:   toggle raw/regex pattern |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| N: search previous           |     | x:   clear slot content       |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| t: jump to timestamp         |     | m:   open finder menu         |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| j: jump down n lines         |     +-------------------------------+"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| J: jump up n lines           |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| b: set bookmark              |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| g: open bookmark menu        |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| ,: undo window vertical move |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| .: redo window vertical move |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("| F: enter follow mode         |"));
    canvas.popup_menu.push(LineWithRenderScheme::new("+------------------------------+"));
}
