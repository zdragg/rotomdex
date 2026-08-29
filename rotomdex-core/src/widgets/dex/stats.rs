use colorgrad::{BasisGradient, Gradient, GradientBuilder};
use ratatui::{
    layout::{Constraint, Flex, Layout},
    widgets::{Bar, BarChart, Widget},
};

use crate::model::{ModelSpecies, ModelVariant};

pub(crate) struct StatsWidget<'a> {
    variant: Option<&'a ModelVariant>,
    species: Option<&'a ModelSpecies>,
}

impl<'a> StatsWidget<'a> {
    pub(crate) fn new(variant: Option<&'a ModelVariant>, species: Option<&'a ModelSpecies>) -> Self {
        Self { variant, species }
    }
}

impl Widget for StatsWidget<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let Some((variant, species)) = self.variant.zip(self.species) else {
            return;
        };
        let highest = species.variants().iter().fold(0, |acc, variant| {
            variant
                .as_loaded()
                .map_or(acc, |variant| acc.max(variant.stats.highest()))
        });

        const BAR_COUNT: u16 = 6;
        const GAP_COUNT: u16 = BAR_COUNT - 1;
        const BAR_TO_GAP_RATIO: u16 = 3;

        if area.width < BAR_COUNT {
            return;
        }

        let total_ratio = u32::from(BAR_COUNT * BAR_TO_GAP_RATIO + GAP_COUNT);
        let width = u32::from(area.width);
        let mut bar_width = ((width * u32::from(BAR_TO_GAP_RATIO) + total_ratio / 2) / total_ratio).max(1) as u16;
        let mut bar_gap = ((width + total_ratio / 2) / total_ratio) as u16;

        if BAR_COUNT * bar_width + GAP_COUNT * bar_gap > area.width {
            if bar_width < BAR_TO_GAP_RATIO * bar_gap {
                bar_gap -= 1;
            } else {
                bar_width -= 1;
            }
        }

        let chart_width = BAR_COUNT * bar_width + GAP_COUNT * bar_gap;
        let [chart_area] = Layout::horizontal([Constraint::Length(chart_width)])
            .flex(Flex::Center)
            .areas(area);

        let bars = vec![
            get_bar(variant.stats.hp, "HP"),
            get_bar(variant.stats.atk, "Atk"),
            get_bar(variant.stats.def, "Def"),
            get_bar(variant.stats.spa, "SpA"),
            get_bar(variant.stats.spd, "SpD"),
            get_bar(variant.stats.spe, "Spe"),
        ];

        BarChart::vertical(bars)
            .max(highest as u64)
            .bar_width(bar_width)
            .bar_gap(bar_gap)
            .render(chart_area, buf);
    }
}

fn get_bar(stat: u32, stat_name: &str) -> Bar<'_> {
    let ratio = (stat as f64 / 180f64).clamp(0.0, 1.0);
    let grad: BasisGradient = GradientBuilder::new()
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
    Bar::with_label(stat_name, stat as u64).style(tui_color)
}
