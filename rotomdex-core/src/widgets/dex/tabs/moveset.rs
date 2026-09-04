use crate::model::{ModelMoveLearnMethod, ModelType, ModelVariant, ModelVersionMove};
use crate::widgets::common::Cursor;
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, HorizontalAlignment, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListState, StatefulWidget, Widget};

pub(super) struct MovesetTabWidget<'a> {
    variant: Option<&'a ModelVariant>,
    state: &'a MovesetTabWidgetState,
}

impl<'a> MovesetTabWidget<'a> {
    pub(super) fn new(variant: Option<&'a ModelVariant>, state: &'a MovesetTabWidgetState) -> Self {
        Self { variant, state }
    }
}

impl<'a> Widget for MovesetTabWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(variant) = self.variant else {
            return;
        };
        let [left, center, right] = area.layout(&Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ]));

        let nonempty_buckets = variant.moves.get_all_nonempty();
        if nonempty_buckets.is_empty() {
            return;
        }
        let bucket_cnt = nonempty_buckets.len();

        let center_idx = self.state.horizontal_cursor.get(bucket_cnt).unwrap();
        let (method, moves) = nonempty_buckets[center_idx];
        render_moves(
            moves,
            method,
            Some(&mut self.state.vertical_cursor.into_list_state(moves.len())),
            HorizontalAlignment::Center,
            center,
            buf,
        );

        if bucket_cnt == 1 {
            Block::bordered().style(Color::DarkGray).render(left, buf);
            Block::bordered().style(Color::DarkGray).render(right, buf);
            return;
        }

        let left_idx = (center_idx + bucket_cnt - 1) % bucket_cnt;
        let (method, moves) = nonempty_buckets[left_idx];
        render_moves(moves, method, None, HorizontalAlignment::Right, left, buf);
        let right_idx = (center_idx + 1) % bucket_cnt;
        let (method, moves) = nonempty_buckets[right_idx];
        render_moves(moves, method, None, HorizontalAlignment::Left, right, buf);
    }
}

// alignment -> text alignment. If on left side, align to right; if on right side, align to left
fn render_moves(
    moves: &[ModelVersionMove],
    method: ModelMoveLearnMethod,
    state: Option<&mut ListState>,
    alignment: HorizontalAlignment,
    area: Rect,
    buf: &mut Buffer,
) {
    let color = match alignment {
        HorizontalAlignment::Center => Color::White,
        _ => Color::DarkGray,
    };

    let block = Block::bordered().style(color).title(method.to_string());
    let list = List::new(moves.iter().map(|move_| get_move_line(move_).alignment(alignment)))
        .block(block)
        .highlight_symbol("> ")
        .scroll_padding(1);

    if let Some(state) = state {
        StatefulWidget::render(list, area, buf, state);
    } else {
        Widget::render(list, area, buf);
    }
}

fn get_move_line(move_: &ModelVersionMove) -> Line<'_> {
    let name = move_.to_string();
    let Some(move_) = move_.resource.as_loaded() else {
        return Line::from(name);
    };
    let type_color = move_.type_.tui_color();
    Line::styled(name, type_color)
}

#[derive(Default)]
pub(super) struct MovesetTabWidgetState {
    horizontal_cursor: Cursor,
    vertical_cursor: Cursor,
}

impl MovesetTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {
        match action {
            TabAction::Right => {
                self.horizontal_cursor.next();
                self.vertical_cursor.reset();
            }
            TabAction::Left => {
                self.horizontal_cursor.prev();
                self.vertical_cursor.reset();
            }
            TabAction::Down => self.vertical_cursor.next(),
            TabAction::Up => self.vertical_cursor.prev(),
            _ => {}
        }
    }
}
