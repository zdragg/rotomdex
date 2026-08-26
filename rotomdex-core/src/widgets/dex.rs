mod ability;
mod name;
mod sprite;
mod stats;
mod variant_selector;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::Line,
    widgets::{Block, Widget},
};

use crate::{
    projector::ProjectorView,
    widgets::dex::{
        ability::AbilitiesWidget, name::NameWidget, sprite::SpriteWidget, stats::StatsWidget,
        variant_selector::VariantSelectorWidget,
    },
};

pub(crate) struct DexWidget<'a> {
    view: ProjectorView<'a>,
    bottom_text: &'a str,
}

impl<'a> DexWidget<'a> {
    pub(crate) fn new(view: ProjectorView<'a>, bottom_text: &'a str) -> Self {
        Self { view, bottom_text }
    }
}

impl<'a> Widget for DexWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_style(if let Some(species) = self.view.border {
                species.get_ratatui_color()
            } else {
                Color::DarkGray
            })
            .title_bottom(
                Line::raw(format!(" {} ", self.bottom_text))
                    .style(Color::DarkGray)
                    .centered(),
            );
        let outer = area;
        let area = block.inner(outer);
        block.render(outer, buf);

        let [left_area, _padding, right_area] =
            Layout::horizontal([Constraint::Percentage(35), Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let [sprite_area, _padding, stats_area] =
            Layout::vertical([Constraint::Percentage(70), Constraint::Length(1), Constraint::Fill(1)]).areas(left_area);

        self.view
            .sprite
            .map(|(sprite, elapsed)| SpriteWidget::new(sprite, elapsed))
            .render(sprite_area, buf);

        self.view
            .stats
            .map(|(variant, species)| StatsWidget::new(variant, species)) // Species has to exist if variant exists
            .render(stats_area, buf);

        let [name_area, variants_area, area] =
            Layout::vertical([Constraint::Percentage(20), Constraint::Length(5), Constraint::Fill(1)])
                .areas(right_area);

        self.view
            .name
            .map(|(species, maybe_variant)| NameWidget::new(species, maybe_variant))
            .render(name_area, buf);

        self.view
            .variant_selector
            .map(|(species, idx)| VariantSelectorWidget::new(species, idx))
            .render(variants_area, buf);

        self.view
            .abilities
            .map(|(variant, idx)| AbilitiesWidget::new(variant, idx))
            .render(area, buf);
    }
}
