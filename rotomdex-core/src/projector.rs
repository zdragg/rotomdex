use std::time::Duration;

use web_time::Instant;

use strum::{EnumIter, IntoEnumIterator};

use crate::{
    Action, ActionHandler,
    model::{ModelPokemon, ModelSpecies, ModelSprite, ModelVariant},
};

/// Projects `[ModelPokemon]` onto the canvas, `[DexWidget]`
pub(crate) struct Model2WidgetProjector {
    timer: Instant,

    focused: Section,
    var_cursor: Cursor,
    abil_cursor: Cursor,
}

impl Model2WidgetProjector {
    pub(crate) fn new() -> Self {
        Self {
            timer: Instant::now(),

            focused: Section::default(),
            var_cursor: Cursor::default(),
            abil_cursor: Cursor::default(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.focused.reset();
        self.var_cursor.reset();
        self.abil_cursor.reset();
    }

    pub(crate) fn project<'a>(&self, pkmn: &'a ModelPokemon) -> ProjectorView<'a> {
        let species = pkmn.species().as_loaded();
        let variant_idx = species.and_then(|species| self.var_cursor.get(species.variants_cnt()));
        let variant = species
            .zip(variant_idx)
            .and_then(|(species, idx)| species.variants().get(idx))
            .and_then(|variant| variant.as_loaded());
        let ability_idx = variant.and_then(|variant| self.abil_cursor.get(variant.abilities().ability_cnt()));
        let sprite = variant.and_then(|v| v.sprite().as_loaded());

        ProjectorView {
            border: species,
            sprite: sprite.map(|sprite| (sprite, self.timer.elapsed())),
            stats: variant.zip(species),
            name: species.map(|species| (species, variant)),
            variant_selector: species.zip(variant_idx),
            abilities: variant.zip(ability_idx),
        }
    }
}

pub(crate) struct ProjectorView<'a> {
    pub(crate) border: Option<&'a ModelSpecies>,
    pub(crate) sprite: Option<(&'a ModelSprite, Duration)>,
    pub(crate) stats: Option<(&'a ModelVariant, &'a ModelSpecies)>,
    pub(crate) name: Option<(&'a ModelSpecies, Option<&'a ModelVariant>)>,
    pub(crate) variant_selector: Option<(&'a ModelSpecies, usize)>,
    pub(crate) abilities: Option<(&'a ModelVariant, usize)>,
}

impl ActionHandler for Model2WidgetProjector {
    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Left | Action::Right => match self.focused {
                Section::VariantSelect => self.var_cursor.handle_action(action),
                Section::Abilities => self.abil_cursor.handle_action(action),
            },
            Action::Down => self.focused.next(),
            Action::Up => self.focused.prev(),
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct Cursor {
    idx: usize,
}

impl Cursor {
    fn reset(&mut self) {
        self.idx = 0;
    }
    fn next(&mut self) {
        self.idx = self.idx.wrapping_add(1);
    }
    fn prev(&mut self) {
        self.idx = self.idx.wrapping_sub(1);
    }
    fn get(&self, total: usize) -> Option<usize> {
        self.idx.checked_rem(total)
    }
}

impl ActionHandler for Cursor {
    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Right => self.next(),
            Action::Left => self.prev(),
            _ => {}
        }
    }
}

#[derive(Default, EnumIter, PartialEq)]
enum Section {
    #[default]
    VariantSelect,
    Abilities,
}

impl Section {
    fn reset(&mut self) {
        *self = Self::default()
    }
    fn next(&mut self) {
        *self = Self::iter()
            .cycle()
            .skip_while(|section| section != self)
            .nth(1)
            .unwrap();
    }

    fn prev(&mut self) {
        *self = Self::iter()
            .rev()
            .cycle()
            .skip_while(|section| section != self)
            .nth(1)
            .unwrap();
    }
}
