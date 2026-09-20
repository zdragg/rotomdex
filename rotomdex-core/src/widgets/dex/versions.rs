use alloc::vec::Vec;
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    macros::constraints,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, HighlightSpacing, List, ListItem, StatefulWidget, Widget},
};
use strum::{EnumCount, VariantArray};

use crate::{Command, DexKeyCode, Version, VersionGroup, data::ModelSpecies, widgets::Cursor};

#[derive(Default)]
pub struct VersionState {
    pub(super) enabled: bool,
    vertical: Cursor,
    horizontal: Cursor,
}

impl VersionState {
    pub(crate) fn toggle(&mut self, current_version: Version) {
        self.enabled = !self.enabled;

        if self.enabled {
            let group = current_version.version_group();
            let group_idx = VersionGroup::VARIANTS
                .iter()
                .position(|candidate| *candidate == group)
                .unwrap();
            self.vertical.select(group_idx as isize);
            let member_idx = group
                .versions()
                .iter()
                .position(|candidate| *candidate == current_version)
                .unwrap();
            self.horizontal.select(member_idx as isize);
        }
    }

    pub(crate) fn handle_key(&mut self, key_code: DexKeyCode, cmd: &mut Option<Command>) {
        match key_code {
            DexKeyCode::Char('j') | DexKeyCode::Down => {
                self.vertical.next();
                self.horizontal.reset();
            }
            DexKeyCode::Char('k') | DexKeyCode::Up => {
                self.vertical.prev();
                self.horizontal.reset();
            }
            DexKeyCode::Char('h') | DexKeyCode::Left => {
                self.horizontal.prev();
            }
            DexKeyCode::Char('l') | DexKeyCode::Right => {
                self.horizontal.next();
            }
            DexKeyCode::Enter => *cmd = Some(Command::NewVersion(self.selected_member())),
            _ => {}
        }
    }

    fn selected_group(&self) -> usize {
        self.vertical.get(VersionGroup::COUNT).unwrap()
    }

    fn selected_versions(&self) -> Vec<Version> {
        VersionGroup::VARIANTS[self.selected_group()].versions()
    }

    fn selected_member(&self) -> Version {
        self.selected_versions()[self.horizontal.get(self.selected_versions().len()).unwrap()]
    }
}

pub struct VersionWidget<'a> {
    species: Option<&'a ModelSpecies>,
    version: Version,
    state: &'a VersionState,
}

impl<'a> VersionWidget<'a> {
    pub(crate) fn new(
        species: Option<&'a ModelSpecies>,
        version: Version,
        state: &'a VersionState,
    ) -> Self {
        Self {
            species,
            version,
            state,
        }
    }
}

const WIDTH: u16 = 16;

impl<'a> Widget for VersionWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.enabled {
            return;
        }

        let color = self
            .species
            .map_or(Color::DarkGray, |species| species.color);

        let [area, _] = area.layout(&Layout::horizontal(constraints![==WIDTH, *=1]));

        let selected_group = self.state.selected_group();
        let items = VersionGroup::VARIANTS
            .iter()
            .enumerate()
            .map(|(group_idx, group)| {
                let mut spans = Vec::new();

                for (version_idx, version) in group.versions().iter().enumerate() {
                    if version_idx != 0 {
                        spans.push(Span::styled(" / ", Color::DarkGray));
                    }

                    let [r, g, b] = version.color();
                    let mut style = Style::default().fg(Color::Rgb(r, g, b));
                    if group_idx == selected_group
                        && version_idx
                            == self
                                .state
                                .horizontal
                                .get(self.state.selected_versions().len())
                                .unwrap()
                    {
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
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(color)
                    .title("Versions"),
            )
            .highlight_symbol("> ")
            .scroll_padding(1)
            .highlight_spacing(HighlightSpacing::Always);

        Clear.render(area, buf);
        StatefulWidget::render(
            list,
            area,
            buf,
            &mut self.state.vertical.list_state(VersionGroup::COUNT),
        );
    }
}
