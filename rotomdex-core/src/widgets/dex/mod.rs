mod keybinds;
mod name;
mod search;
mod sprite;
mod stats;
mod tabs;
mod variant;
mod versions;

use crate::data::DataStore;
use crate::data::resource::AsyncResource;
use crate::widgets::dex::keybinds::{TutorialWidget, TutorialWidgetState};
use crate::widgets::dex::search::{SearchWidget, SearchWidgetState};
use crate::widgets::dex::sprite::SpriteWidgetState;
use crate::widgets::dex::tabs::TabsWidgetState;
use crate::widgets::dex::versions::{VersionState, VersionWidget};
use crate::widgets::{Cursor, RenderBlockExt};
use crate::{Command, Settings};
use crate::{
    DexKeyCode,
    widgets::dex::{
        name::NameWidget, sprite::SpriteWidget, stats::StatsWidget, tabs::TabsWidget,
        variant::VariantSelectorWidget,
    },
};

use alloc::string::ToString;
use embassy_time::Duration;
use ratatui::macros::constraints;
use ratatui::widgets::BorderType;
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::Color,
    widgets::{Block, Widget},
};

pub(crate) struct DexWidget<'a> {
    data: &'a DataStore,
    elapsed: Duration,
    settings: Settings,

    state: &'a DexWidgetState,
}

impl<'a> DexWidget<'a> {
    pub(crate) fn new(
        data: &'a DataStore,
        state: &'a DexWidgetState,
        settings: Settings,
        elapsed: Duration,
    ) -> Self {
        Self {
            data,
            elapsed,
            settings,
            state,
        }
    }
}

impl Widget for DexWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let species = self.data.resource.as_loaded();

        let variant_idx =
            species.and_then(|species| self.state.variant_cursor.get(species.variants.len()));
        let variant = species
            .zip(variant_idx)
            .and_then(|(species, idx)| species.variants.get(idx))
            .and_then(|variant| variant.as_loaded());

        // Block + bottom text / search widget render
        let [_area, bottom_text_area] = area.layout(&Layout::vertical(constraints![*=1, ==1]));

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(species.map_or(Color::DarkGray, |species| species.color));
        let area = block.render_inner(area, buf);

        let displayed_error = if let AsyncResource::Failed(error) = &self.data.resource {
            Some(error.to_string())
        } else {
            None
        };

        SearchWidget::new(&self.state.search_state, displayed_error).render(bottom_text_area, buf); // Overlay the search widget on top of the block

        let [left_area, right_area] = Layout::horizontal(constraints![==35%, *=1])
            .spacing(1)
            .areas(area);
        let [sprite_area, stats_area] = Layout::vertical(constraints![==70%, *=1])
            .spacing(1)
            .areas(left_area);
        let [name_area, variants_area, tab_area] =
            Layout::vertical(constraints![==20%, ==2, *=1]).areas(right_area);

        SpriteWidget::new(
            variant,
            self.elapsed,
            self.settings.animation_mode,
            &self.state.sprite_state,
        )
        .render(sprite_area, buf);
        StatsWidget::new(species, variant).render(stats_area, buf);
        NameWidget::new(species, variant).render(name_area, buf);
        VariantSelectorWidget::new(species, variant_idx).render(variants_area, buf);
        TabsWidget::new(species, variant, &self.state.tabs_state).render(tab_area, buf);
        TutorialWidget::new(species, &self.state.tutorial_state).render(area, buf);
        VersionWidget::new(species, self.settings.version, &self.state.version_state)
            .render(stats_area, buf);
    }
}

#[derive(Default)]
pub(crate) struct DexWidgetState {
    variant_cursor: Cursor,

    version_state: VersionState,
    search_state: SearchWidgetState,
    tabs_state: TabsWidgetState,
    tutorial_state: TutorialWidgetState,
    pub(crate) sprite_state: SpriteWidgetState,
}

impl DexWidgetState {
    pub(crate) fn handle_key(
        &mut self,
        key_code: DexKeyCode,
        settings: Settings,
        cmd: &mut Option<Command>,
    ) {
        match key_code {
            DexKeyCode::Char('.') => {
                self.version_state.toggle(settings.version);
                return;
            }
            DexKeyCode::Char(':') => {
                self.search_state.start_search();
                return;
            }
            DexKeyCode::Char('/') => {
                self.tutorial_state.enabled = !self.tutorial_state.enabled;
                return;
            }
            _ => {}
        }

        if self.version_state.enabled {
            self.version_state.handle_key(key_code, cmd);
            return;
        }

        if self.search_state.searching {
            self.search_state.handle_key(key_code, cmd);
            return;
        }

        match key_code {
            DexKeyCode::Char('f') => self.variant_cursor.next(),
            DexKeyCode::Char('d') => self.variant_cursor.prev(),
            _ => self.tabs_state.handle_key(key_code, cmd),
        };
    }
}
