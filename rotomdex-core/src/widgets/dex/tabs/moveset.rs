use crate::InnerActionResult;
use crate::model::{ModelDamageClass, ModelMoveLearnMethod, ModelVariant, ModelVersionMove};
use crate::widgets::RenderBlockExt;
use crate::widgets::common::Cursor;
use crate::widgets::dex::tabs::TabAction;
use itertools::Itertools;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, HorizontalAlignment, Layout, Margin, Rect};
use ratatui::macros::constraints;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, ToLine, ToSpan};
use ratatui::widgets::{Block, Clear, List, ListState, Padding, Paragraph, StatefulWidget, Widget, Wrap};

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
        let areas: [Rect; 3] = area.layout(&Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(4),
            Constraint::Fill(1),
        ]));

        let Some(variant) = self.variant else {
            areas.into_iter().for_each(|area| Block::bordered().render(area, buf));
            return;
        };

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
            areas[1],
            buf,
        );

        if bucket_cnt == 1 {
            Block::bordered().style(Color::DarkGray).render(areas[0], buf);
            Block::bordered().style(Color::DarkGray).render(areas[2], buf);
            return;
        }

        let left_idx = (center_idx + bucket_cnt - 1) % bucket_cnt;
        let (left_method, left_moves) = nonempty_buckets[left_idx];
        render_left(left_moves, left_method.to_line(), areas[0], buf);

        let right_idx = (center_idx + 1) % bucket_cnt;
        let (right_method, right_moves) = nonempty_buckets[right_idx];
        render_right(right_moves, right_method.to_line(), areas[2], buf);

        // Render details overlaid on everything else if enabled
        if self.state.move_detail_mode {
            let selected_move = &moves[self.state.vertical_cursor.get(moves.len()).unwrap()];
            render_details(selected_move, area, buf);
        }
    }
}

fn render_center(
    moves: &[ModelVersionMove],
    method: ModelMoveLearnMethod,
    state: &mut ListState,
    area: Rect,
    buf: &mut Buffer,
) {
    let block = Block::bordered().title(method.to_line());
    let item_width = area.width.saturating_sub(4) as usize;
    let moves = moves
        .into_iter()
        .map(|move_| move_line(move_, item_width, method).alignment(HorizontalAlignment::Center));
    let list = List::new(moves).highlight_symbol(">").block(block).scroll_padding(1);

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
                machine.to_span()
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

fn render_details(move_: &ModelVersionMove, area: Rect, buf: &mut Buffer) {
    let area = area.inner(Margin::new(1, 1));
    Clear.render(area, buf);
    let area = Block::bordered()
        .padding(Padding::symmetric(3, 1))
        .render_inner(area, buf);

    let Some(move_) = move_.resource.as_loaded() else {
        Span::styled(format!("loading {}...", move_.name), Color::DarkGray).render(area, buf);
        return;
    };

    let [name_area, info_area, _, text_area] = area.layout(&Layout::vertical(constraints![==1, ==1, ==1, *=1]));

    let name_style = Style::from(move_.type_.tui_color());
    let name_style = match move_.damage_class {
        ModelDamageClass::Physical => name_style.bold(),
        ModelDamageClass::Special => name_style.italic(),
        ModelDamageClass::Status => name_style.underlined(),
    };
    let name_span = Span::styled(move_.name.clone(), name_style);

    let damage_class_span = Span::styled(format!(" ({}) ", move_.damage_class), Color::DarkGray);

    let effectiveness_spans = move_.type_.atk_effectiveness().into_spans();

    Line::from_iter(itertools::chain!([name_span], [damage_class_span], effectiveness_spans)).render(name_area, buf);

    let power = move_.power.map_or("bp: N/A".into(), |x| format!("bp: {x}"));
    let acc = move_.accuracy.map_or("acc: N/A".into(), |x| format!("acc: {x}%"));
    let chance = move_
        .effect_chance
        .map_or("effect: N/A".into(), |x| format!("effect: {x}%"));
    let info_span = Span::raw(format!("{power} {acc} {chance}"));
    info_span.render(info_area, buf);

    if let Some(text) = move_.short_effect.as_ref().or(move_.effect.as_ref()) {
        Paragraph::new(text.as_str())
            .gray()
            .wrap(Wrap { trim: true })
            .render(text_area, buf);
    }
}

#[derive(Default)]
pub(super) struct MovesetTabWidgetState {
    horizontal_cursor: Cursor,
    vertical_cursor: Cursor,

    move_detail_mode: bool,
}

impl MovesetTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) -> InnerActionResult {
        if !self.move_detail_mode {
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
                TabAction::Enter => self.move_detail_mode = !self.move_detail_mode,
                _ => {}
            }
        } else {
            match action {
                TabAction::Enter | TabAction::Escape => self.move_detail_mode = !self.move_detail_mode,
                _ => {}
            }
        }
        InnerActionResult::Nothing
    }
}
