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
    widgets::{
        OptionWidgetExt,
        dex::{
            ability::AbilitiesWidget, name::NameWidget, sprite::SpriteWidget, stats::StatsWidget,
            variant_selector::VariantSelectorWidget,
        },
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

        let [left_area, right_area] = Layout::horizontal([Constraint::Percentage(35), Constraint::Fill(1)])
            .spacing(1)
            .areas(area);

        let [sprite_area, stats_area] = Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)])
            .spacing(1)
            .areas(left_area);

        self.view.sprite.render_option::<SpriteWidget>(sprite_area, buf);

        self.view.stats.render_option::<StatsWidget>(stats_area, buf);

        let [name_area, variants_area, abilities_area, area] = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Percentage(5),
            Constraint::Percentage(20),
            Constraint::Fill(1),
        ])
        .areas(right_area);

        self.view.name.render_option::<NameWidget>(name_area, buf);

        self.view
            .variant_selector
            .render_option::<VariantSelectorWidget>(variants_area, buf);

        self.view
            .abilities
            .render_option::<AbilitiesWidget>(abilities_area, buf);
    }
}
