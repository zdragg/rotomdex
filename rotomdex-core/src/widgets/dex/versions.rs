use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, HighlightSpacing, List, ListItem, StatefulWidget, Widget},
};
use strum::{EnumCount, VariantArray};

use crate::{DexKeyCode, InnerActionResult, Version, VersionGroup, widgets::Cursor};

#[derive(Default)]
pub struct VersionState {
    pub(super) enabled: bool,
    cursor: Cursor,
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
            self.cursor.select(group_idx as isize);
            self.horizontal = group
                .versions()
                .iter()
                .position(|candidate| *candidate == version)
                .unwrap();
        }
    }

    pub(crate) fn handle_key(&mut self, key_code: DexKeyCode) -> InnerActionResult {
        match key_code {
            DexKeyCode::Char('j') | DexKeyCode::Down => {
                self.cursor.next();
                self.horizontal = 0;
            }
            DexKeyCode::Char('k') | DexKeyCode::Up => {
                self.cursor.prev();
                self.horizontal = 0;
            }
            DexKeyCode::Char('h') | DexKeyCode::Left => {
                let count = self.selected_versions().len();
                self.horizontal = (self.horizontal + count - 1) % count;
            }
            DexKeyCode::Char('l') | DexKeyCode::Right => {
                self.horizontal = (self.horizontal + 1) % self.selected_versions().len();
            }
            DexKeyCode::Enter => return InnerActionResult::NewVersion(self.selected_versions()[self.horizontal]),
            _ => {}
        }
        InnerActionResult::Nothing
    }

    fn selected_group(&self) -> usize {
        self.cursor.get(VersionGroup::COUNT).unwrap()
    }

    fn selected_versions(&self) -> Vec<Version> {
        VersionGroup::VARIANTS[self.selected_group()].versions()
    }
}

pub struct VersionWidget<'a> {
    version: Version,
    state: &'a VersionState,
}

impl<'a> VersionWidget<'a> {
    pub(crate) fn new(version: Version, state: &'a VersionState) -> Self {
        Self { version, state }
    }
}

impl<'a> Widget for VersionWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.enabled {
            return;
        }

        let [area, _] = area.layout(&Layout::horizontal([Constraint::Length(16), Constraint::Fill(1)]));

        let selected_group = self.state.selected_group();
        let items = VersionGroup::VARIANTS.iter().enumerate().map(|(group_idx, group)| {
            let mut spans = Vec::new();

            for (version_idx, version) in group.versions().iter().enumerate() {
                if version_idx != 0 {
                    spans.push(Span::styled(" / ", Color::DarkGray));
                }

                let [r, g, b] = version.color();
                let mut style = Style::default().fg(Color::Rgb(r, g, b));
                if group_idx == selected_group && version_idx == self.state.horizontal {
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
            .scroll_padding(1)
            .highlight_spacing(HighlightSpacing::Always);

        Clear.render(area, buf);
        StatefulWidget::render(list, area, buf, &mut self.state.cursor.list_state(VersionGroup::COUNT));
    }
}
