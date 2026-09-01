use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};
use strum::{EnumCount, VariantArray};

use crate::{Action, ActionResult, Version, VersionGroup};

#[derive(Default)]
pub struct VersionState {
    pub(super) enabled: bool,
    list: ListState,
    horizontal: usize,
}

impl VersionState {
    pub(crate) fn toggle(&mut self, version: Version) {
        self.enabled = !self.enabled;
        self.horizontal = 0;

        if self.enabled {
            let group = version.version_group();
            let group_idx = VersionGroup::VARIANTS
                .iter()
                .position(|candidate| *candidate == group)
                .unwrap();
            self.list.select(Some(group_idx));
            self.horizontal = group
                .versions()
                .iter()
                .position(|candidate| *candidate == version)
                .unwrap();
        }
    }

    pub(crate) fn handle_action(&mut self, action: Action) -> ActionResult {
        match action {
            Action::Input('j') | Action::Down => {
                let selected = (self.selected_group() + 1) % VersionGroup::COUNT;
                self.list.select(Some(selected));
                self.horizontal = 0;
            }
            Action::Input('k') | Action::Up => {
                let selected = (self.selected_group() + VersionGroup::COUNT - 1) % VersionGroup::COUNT;
                self.list.select(Some(selected));
                self.horizontal = 0;
            }
            Action::Input('h') | Action::Left => {
                let count = self.selected_versions().len();
                self.horizontal = (self.horizontal + count - 1) % count;
            }
            Action::Input('l') | Action::Right => {
                self.horizontal = (self.horizontal + 1) % self.selected_versions().len();
            }
            Action::Enter => return ActionResult::NewVersion(self.selected_versions()[self.horizontal]),
            _ => {}
        }
        ActionResult::Nothing
    }

    fn selected_group(&self) -> usize {
        self.list.selected().unwrap()
    }

    fn selected_versions(&self) -> Vec<Version> {
        VersionGroup::VARIANTS[self.selected_group()].versions()
    }
}

pub struct VersionWidget {
    version: Version,
}

impl VersionWidget {
    pub(crate) fn new(version: Version) -> Self {
        Self { version }
    }
}

impl StatefulWidget for VersionWidget {
    type State = VersionState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.enabled {
            return;
        }

        let [area, _] = area.layout(&Layout::horizontal([Constraint::Length(16), Constraint::Fill(1)]));

        let selected_group = state.selected_group();
        let items = VersionGroup::VARIANTS.iter().enumerate().map(|(group_idx, group)| {
            let mut spans = Vec::new();

            for (version_idx, version) in group.versions().iter().enumerate() {
                if version_idx != 0 {
                    spans.push(Span::styled(" / ", Color::DarkGray));
                }

                let [r, g, b, _] = csscolorparser::parse(version.color()).unwrap().to_rgba8();
                let mut style = Style::default().fg(Color::Rgb(r, g, b));
                if group_idx == selected_group && version_idx == state.horizontal {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                spans.push(Span::styled(version.abbreviation(), style));

                if *version == self.version {
                    spans.push(Span::styled(" ✓", Color::Green));
                }
            }

            ListItem::new(Line::from(spans))
        });

        let list = List::new(items)
            .block(Block::bordered().title("Versions").title_alignment(Alignment::Left))
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        Clear.render(area, buf);
        StatefulWidget::render(list, area, buf, &mut state.list);
    }
}
