use crate::model::{ModelDamageClass, ModelMoveLearnMethod, ModelVariant, ModelVersionMove};
use crate::widgets::common::Cursor;
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, HorizontalAlignment, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, ToLine};
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
            Constraint::Fill(4),
            Constraint::Fill(1),
        ]));

        let nonempty_buckets = variant.moves.get_all_nonempty();
        if nonempty_buckets.is_empty() {
            return;
        }
        let bucket_cnt = nonempty_buckets.len();

        let center_idx = self.state.horizontal_cursor.get(bucket_cnt).unwrap();
        let (method, moves) = nonempty_buckets[center_idx];
        render_center(
            moves,
            method,
            &mut self.state.vertical_cursor.list_state(moves.len()),
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
        render_left(moves, method.to_line(), left, buf);

        let right_idx = (center_idx + 1) % bucket_cnt;
        let (method, moves) = nonempty_buckets[right_idx];
        render_right(moves, method.to_line(), right, buf);
    }
}

fn render_center(
    moves: &[ModelVersionMove],
    method: ModelMoveLearnMethod,
    state: &mut ListState,
    area: Rect,
    buf: &mut Buffer,
) {
    let block = Block::bordered().style(Color::White).title(method.to_line());
    let item_width = area.width.saturating_sub(4) as usize;
    let list = List::new(
        moves
            .iter()
            .map(|move_| move_line(move_, item_width, method).alignment(HorizontalAlignment::Center)),
    )
    .highlight_symbol(">")
    .block(block)
    .scroll_padding(1);

    StatefulWidget::render(list, area, buf, state);
}

fn render_left(moves: &[ModelVersionMove], title: Line, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered().style(Color::DarkGray).title(title);
    let list = List::new(
        moves
            .iter()
            .map(|move_| short_move_line(move_).alignment(HorizontalAlignment::Right)),
    )
    .block(block);

    Widget::render(list, area, buf);
}

fn render_right(moves: &[ModelVersionMove], title: Line, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered().style(Color::DarkGray).title(title);
    let list = List::new(
        moves
            .iter()
            .map(|move_| short_move_line(move_).alignment(HorizontalAlignment::Left)),
    )
    .block(block);

    Widget::render(list, area, buf);
}

fn move_line(move_: &ModelVersionMove, width: usize, method: ModelMoveLearnMethod) -> Line<'_> {
    let level_learned_at = move_.level_learned_at;
    let Some(move_) = move_.resource.as_loaded() else {
        return short_move_line(move_);
    };

    let left = match method {
        ModelMoveLearnMethod::LevelUp => Span::raw(format!("lv{}", level_learned_at)),
        ModelMoveLearnMethod::Machine => {
            if let Some(machine) = move_.machine.as_ref().map(|resource| resource.as_loaded()).flatten() {
                Span::raw(&machine.name)
            } else {
                Span::default()
            }
        }
        _ => Span::default(),
    };

    let color = move_.type_.tui_color();
    let modifier = match move_.damage_class {
        ModelDamageClass::Physical => Modifier::BOLD,
        ModelDamageClass::Special => Modifier::ITALIC,
        ModelDamageClass::Status => Modifier::UNDERLINED,
    };
    let center = Span::styled(move_.name.as_str(), Style::default().patch(color).patch(modifier));

    let power = move_.power.map_or("_".to_string(), |x| x.to_string());
    let accuracy = move_.accuracy.map_or("_".to_string(), |x| x.to_string());
    let right = Span::raw(format!("{}/{}%", power, accuracy));

    merge_spans(left, center, right, width)
}

fn short_move_line(move_: &ModelVersionMove) -> Line<'_> {
    Line::styled(move_.name.as_str(), Color::DarkGray)
}

fn merge_spans<'a>(
    left: impl Into<Line<'a>>,
    center: impl Into<Line<'a>>,
    right: impl Into<Line<'a>>,
    width: usize,
) -> Line<'a> {
    let left = left.into().spans;
    let center = center.into().spans;
    let right = right.into().spans;
    let span_width = |spans: &[Span<'_>]| spans.iter().map(|span| span.content.chars().count()).sum::<usize>();

    let left_width = span_width(&left);
    let center_width = span_width(&center);
    let right_width = span_width(&right);
    let center_gap = (width.saturating_sub(center_width) / 2).saturating_sub(left_width);
    let center_end = left_width + center_gap + center_width;
    let right_gap = width.saturating_sub(right_width).saturating_sub(center_end);

    let mut spans = Vec::with_capacity(left.len() + center.len() + right.len() + 2);
    spans.extend(left);
    spans.push(Span::raw(" ".repeat(center_gap)));
    spans.extend(center);
    spans.push(Span::raw(" ".repeat(right_gap)));
    spans.extend(right);

    Line::from(spans)
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
