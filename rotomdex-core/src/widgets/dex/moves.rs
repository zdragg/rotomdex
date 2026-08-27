use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::{
    model::{ModelMoveLearnMethod, ModelVersionMove},
    projector::Section,
    widgets::WidgetExt,
};

pub(crate) struct MovesWidget<'a> {
    moves: &'a [ModelVersionMove],
    selected_move: usize,
    focused: bool,
}

impl<'a> WidgetExt<(&'a [ModelVersionMove], usize, Section)> for MovesWidget<'a> {
    fn new((moves, selected_move, section): (&'a [ModelVersionMove], usize, Section)) -> Self {
        Self {
            moves,
            selected_move,
            focused: section == Section::Moves,
        }
    }
}

impl<'a> Widget for MovesWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [lvlup_area, machine_area, egg_area, tutor_area] = area.layout(&Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]));

        let lvlup_moves = self
            .moves
            .iter()
            .enumerate()
            .filter(|(_, move_)| matches!(move_.learn_method, ModelMoveLearnMethod::LevelUp(_)));
        let machine_moves = self
            .moves
            .iter()
            .enumerate()
            .filter(|(_, move_)| matches!(move_.learn_method, ModelMoveLearnMethod::Machine));
        let egg_moves = self
            .moves
            .iter()
            .enumerate()
            .filter(|(_, move_)| matches!(move_.learn_method, ModelMoveLearnMethod::Egg));
        let tutor_moves = self
            .moves
            .iter()
            .enumerate()
            .filter(|(_, move_)| matches!(move_.learn_method, ModelMoveLearnMethod::Tutor));

        render_moves(
            "level-up",
            lvlup_moves,
            self.selected_move,
            self.focused,
            lvlup_area,
            buf,
        );
        render_moves(
            "machine",
            machine_moves,
            self.selected_move,
            self.focused,
            machine_area,
            buf,
        );
        render_moves("egg", egg_moves, self.selected_move, self.focused, egg_area, buf);
        render_moves("tutor", tutor_moves, self.selected_move, self.focused, tutor_area, buf);
    }
}

fn render_moves<'a>(
    heading: &str,
    moves: impl Iterator<Item = (usize, &'a ModelVersionMove)>,
    selected_move: usize,
    focused: bool,
    area: Rect,
    buf: &mut ratatui::prelude::Buffer,
) {
    let mut lines = vec![Line::raw(heading)];
    lines.extend(moves.map(|(idx, move_)| {
        let marker = if idx == selected_move {
            if focused { ">" } else { "*" }
        } else {
            " "
        };
        let level = match move_.learn_method {
            ModelMoveLearnMethod::LevelUp(level) => format!(" lv.{level}"),
            _ => String::new(),
        };
        Line::raw(format!("{marker} {}{level}", move_.name))
    }));
    Paragraph::new(lines).render(area, buf);
}
