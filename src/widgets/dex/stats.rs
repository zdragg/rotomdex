use colorgrad::{BasisGradient, Gradient, GradientBuilder};
use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::Color,
    symbols,
    widgets::{LineGauge, Widget},
};

use crate::offline::OfflineStats;

pub struct StatsWidget<'a> {
    stats: &'a OfflineStats,
}

impl<'a> StatsWidget<'a> {
    pub fn new(stats: &'a OfflineStats) -> Self {
        Self { stats }
    }
}

impl<'a> Widget for StatsWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [offensive, _padding, defensive] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        let [atk, spa, spe] = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(offensive);
        let [def, spd, hp] = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(defensive);

        StatWidget::new(self.stats.atk, "atk").render(atk, buf);
        StatWidget::new(self.stats.spa, "spa").render(spa, buf);
        StatWidget::new(self.stats.spe, "spe").render(spe, buf);
        StatWidget::new(self.stats.def, "def").render(def, buf);
        StatWidget::new(self.stats.spd, "spd").render(spd, buf);
        StatWidget::new(self.stats.hp, "hp").render(hp, buf);
    }
}

pub struct StatWidget<'a> {
    stat: u32,
    stat_name: &'a str,
}

impl<'a> StatWidget<'a> {
    pub fn new(stat: u32, stat_name: &'a str) -> Self {
        Self { stat, stat_name }
    }
}

impl<'a> Widget for StatWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let ratio = (self.stat as f64 / 180f64).clamp(0.0, 1.0);
        let grad: BasisGradient = GradientBuilder::new()
            // Colors inspired from pokemondb.net
            .html_colors(&["#f34444", "#ff7f0f", "#ffdd57", "#a0e515", "#23cd5e", "#00c2b8"])
            .mode(colorgrad::BlendMode::Oklab)
            .build()
            .unwrap();
        let color = grad.at(ratio as f32).clamp();
        let tui_color = ratatui::style::Color::Rgb(
            (color.r * 255.0).round() as u8,
            (color.g * 255.0).round() as u8,
            (color.b * 255.0).round() as u8,
        );
        LineGauge::default()
            .label(format!("{:>3}: {:<3}", self.stat_name, self.stat))
            .unfilled_style(Color::DarkGray)
            .filled_style(tui_color)
            .filled_symbol(symbols::line::THICK_HORIZONTAL)
            .ratio(ratio)
            .render(area, buf);
    }
}
