use alloc::{string::String, vec::Vec};

use super::{
    berries::BerryFlavor,
    evolution::EvolutionChain,
    games::{Generation, Pokedex, Version, VersionGroup},
    items::Item,
    locations::{LocationArea, PalParkArea},
    moves::{Move, MoveBattleStyle, MoveDamageClass, MoveLearnMethod},
    resource::{
        ApiResource, Description, Effect, FlavorText, GenerationGameIndex, Name, NamedApiResource,
        VerboseEffect, VersionEncounterDetail, VersionGameIndex,
    },
    utility::Language,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Ability {
    pub id: i64,
    pub name: String,
    pub is_main_series: bool,
    pub generation: NamedApiResource<Generation>,
    pub names: Vec<Name>,
    pub effect_entries: Vec<VerboseEffect>,
    pub effect_changes: Vec<AbilityEffectChange>,
    pub flavor_text_entries: Vec<AbilityFlavorText>,
    pub pokemon: Vec<AbilityPokemon>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct AbilityEffectChange {
    pub effect_entries: Vec<Effect>,
    pub version_group: NamedApiResource<VersionGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct AbilityFlavorText {
    pub flavor_text: String,
    pub language: NamedApiResource<Language>,
    pub version_group: NamedApiResource<VersionGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct AbilityPokemon {
    pub is_hidden: bool,
    pub slot: i64,
    pub pokemon: NamedApiResource<Pokemon>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Characteristic {
    pub id: i64,
    pub gene_modulo: i64,
    pub possible_values: Vec<i64>,
    pub descriptions: Vec<Description>,
    pub highest_stat: NamedApiResource<Stat>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EggGroup {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub pokemon_species: Vec<NamedApiResource<PokemonSpecies>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Gender {
    pub id: i64,
    pub name: String,
    pub pokemon_species_details: Vec<PokemonSpeciesGender>,
    pub required_for_evolution: Vec<NamedApiResource<PokemonSpecies>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonSpeciesGender {
    pub rate: i64,
    pub pokemon_species: NamedApiResource<PokemonSpecies>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GrowthRate {
    pub id: i64,
    pub name: String,
    pub formula: String,
    pub descriptions: Vec<Description>,
    pub levels: Vec<GrowthRateExperienceLevel>,
    pub pokemon_species: Vec<NamedApiResource<PokemonSpecies>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GrowthRateExperienceLevel {
    pub level: i64,
    pub experience: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Nature {
    pub id: i64,
    pub name: String,
    pub decreased_stat: Option<NamedApiResource<Stat>>,
    pub increased_stat: Option<NamedApiResource<Stat>>,
    pub hates_flavor: Option<NamedApiResource<BerryFlavor>>,
    pub likes_flavor: Option<NamedApiResource<BerryFlavor>>,
    pub pokeathlon_stat_changes: Vec<NatureStatChange>,
    pub move_battle_style_preferences: Vec<MoveBattleStylePreference>,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct NatureStatChange {
    pub max_change: i64,
    pub pokeathlon_stat: NamedApiResource<PokeathlonStat>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveBattleStylePreference {
    pub low_hp_preference: i64,
    pub high_hp_preference: i64,
    pub move_battle_style: NamedApiResource<MoveBattleStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokeathlonStat {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub affecting_natures: NaturePokeathlonStatAffectSets,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct NaturePokeathlonStatAffectSets {
    pub increase: Vec<NaturePokeathlonStatAffect>,
    pub decrease: Vec<NaturePokeathlonStatAffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct NaturePokeathlonStatAffect {
    pub max_change: i64,
    pub nature: NamedApiResource<Nature>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Pokemon {
    pub id: i64,
    pub name: String,
    pub base_experience: Option<i64>,
    pub height: i64,
    pub is_default: bool,
    pub order: i64,
    pub weight: i64,
    pub abilities: Vec<PokemonAbility>,
    pub forms: Vec<NamedApiResource<PokemonForm>>,
    pub game_indices: Vec<VersionGameIndex>,
    pub held_items: Vec<PokemonHeldItem>,
    pub location_area_encounters: String,
    pub moves: Vec<PokemonMove>,
    pub past_abilities: Vec<PokemonAbilityPast>,
    pub past_stats: Vec<PokemonStatPast>,
    pub past_types: Vec<PokemonTypePast>,
    pub sprites: PokemonSprites,
    pub cries: PokemonCries,
    pub species: NamedApiResource<PokemonSpecies>,
    pub stats: Vec<PokemonStat>,
    pub types: Vec<PokemonType>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonAbility {
    pub is_hidden: bool,
    pub slot: i64,
    pub ability: Option<NamedApiResource<Ability>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonAbilityPast {
    pub generation: NamedApiResource<Generation>,
    pub abilities: Vec<PokemonAbility>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonType {
    pub slot: i64,
    #[serde(rename = "type")]
    pub type_: NamedApiResource<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonTypePast {
    pub generation: NamedApiResource<Generation>,
    pub types: Vec<PokemonType>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonHeldItem {
    pub item: NamedApiResource<Item>,
    pub version_details: Vec<PokemonHeldItemVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonHeldItemVersion {
    pub version: NamedApiResource<Version>,
    pub rarity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonMove {
    #[serde(rename = "move")]
    pub move_: NamedApiResource<Move>,
    pub version_group_details: Vec<PokemonMoveVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonMoveVersion {
    pub move_learn_method: NamedApiResource<MoveLearnMethod>,
    pub version_group: NamedApiResource<VersionGroup>,
    pub level_learned_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonStat {
    pub stat: NamedApiResource<Stat>,
    pub effort: i64,
    pub base_stat: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonStatPast {
    pub generation: NamedApiResource<Generation>,
    pub stats: Vec<PokemonStat>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonSprites {
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny_female: Option<String>,
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny_female: Option<String>,
    pub other: OtherSprites,
    pub versions: VersionsSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct OtherSprites {
    pub dream_world: DreamWorldSprites,
    pub home: HomeSprites,
    #[serde(rename = "official-artwork")]
    pub official_artwork: OfficialArtworkSprites,
    pub showdown: ShowdownSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct DreamWorldSprites {
    pub front_default: Option<String>,
    pub front_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct HomeSprites {
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct OfficialArtworkSprites {
    pub front_default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ShowdownSprites {
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct VersionsSprites {
    #[serde(rename = "generation-i")]
    pub generation_i: GenerationISprites,
    #[serde(rename = "generation-ii")]
    pub generation_ii: GenerationIISprites,
    #[serde(rename = "generation-iii")]
    pub generation_iii: GenerationIIISprites,
    #[serde(rename = "generation-iv")]
    pub generation_iv: GenerationIVSprites,
    #[serde(rename = "generation-v")]
    pub generation_v: GenerationVSprites,
    #[serde(rename = "generation-vi")]
    pub generation_vi: GenerationVISprites,
    #[serde(rename = "generation-vii")]
    pub generation_vii: GenerationVIISprites,
    #[serde(rename = "generation-viii")]
    pub generation_viii: GenerationVIIISprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationISprites {
    #[serde(rename = "red-blue")]
    pub red_blue: RedBlueSprites,
    pub yellow: YellowSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RedBlueSprites {
    pub back_default: Option<String>,
    pub back_gray: Option<String>,
    pub back_transparent: Option<String>,
    pub front_default: Option<String>,
    pub front_gray: Option<String>,
    pub front_transparent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct YellowSprites {
    pub back_default: Option<String>,
    pub back_gray: Option<String>,
    pub back_transparent: Option<String>,
    pub front_default: Option<String>,
    pub front_gray: Option<String>,
    pub front_transparent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationIISprites {
    pub crystal: CrystalSprites,
    pub gold: GoldSprites,
    pub silver: SilverSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct CrystalSprites {
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub back_transparent: Option<String>,
    pub back_shiny_transparent: Option<String>,
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
    pub front_transparent: Option<String>,
    pub front_shiny_transparent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GoldSprites {
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
    pub front_transparent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct SilverSprites {
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
    pub front_transparent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationIIISprites {
    pub emerald: EmeraldSprites,
    #[serde(rename = "firered-leafgreen")]
    pub firered_leafgreen: FireredLeafgreenSprites,
    #[serde(rename = "ruby-sapphire")]
    pub ruby_sapphire: RubySapphireSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EmeraldSprites {
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct FireredLeafgreenSprites {
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RubySapphireSprites {
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationIVSprites {
    #[serde(rename = "diamond-pearl")]
    pub diamond_pearl: DiamondPearlSprites,
    pub platinum: PlatinumSprites,
    #[serde(rename = "heartgold-soulsilver")]
    pub heartgold_soulsilver: HeartgoldSoulsilverSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct DiamondPearlSprites {
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PlatinumSprites {
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct HeartgoldSoulsilverSprites {
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationVSprites {
    #[serde(rename = "black-white")]
    pub black_white: BlackWhiteSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct BlackWhiteSprites {
    pub animated: BlackWhiteAnimatedSprites,
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct BlackWhiteAnimatedSprites {
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationVISprites {
    #[serde(rename = "omegaruby-alphasapphire")]
    pub omegaruby_alphasapphire: OmegarubyAlphasapphireSprites,
    #[serde(rename = "x-y")]
    pub x_y: XYSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct OmegarubyAlphasapphireSprites {
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct XYSprites {
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationVIISprites {
    pub icons: IconsSprites,
    #[serde(rename = "ultra-sun-ultra-moon")]
    pub ultrasun_ultramoon: UltrasunUltramoonSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct IconsSprites {
    pub front_default: Option<String>,
    pub front_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct UltrasunUltramoonSprites {
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationVIIISprites {
    pub icons: IconsSprites,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonCries {
    pub latest: Option<String>,
    pub legacy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct LocationAreaEncounter {
    pub location_area: NamedApiResource<LocationArea>,
    pub version_details: Vec<VersionEncounterDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonColor {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub pokemon_species: Vec<NamedApiResource<PokemonSpecies>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonForm {
    pub id: i64,
    pub name: String,
    pub order: i64,
    pub form_order: i64,
    pub is_default: bool,
    pub is_battle_only: bool,
    pub is_mega: bool,
    pub form_name: String,
    pub pokemon: NamedApiResource<Pokemon>,
    pub types: Vec<PokemonFormType>,
    pub sprites: PokemonFormSprites,
    pub version_group: NamedApiResource<VersionGroup>,
    pub names: Vec<Name>,
    pub form_names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonFormType {
    pub slot: i64,
    #[serde(rename = "type")]
    pub type_: NamedApiResource<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonFormSprites {
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonHabitat {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub pokemon_species: Vec<NamedApiResource<PokemonSpecies>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonShape {
    pub id: i64,
    pub name: String,
    pub awesome_names: Vec<AwesomeName>,
    pub names: Vec<Name>,
    pub pokemon_species: Vec<NamedApiResource<PokemonSpecies>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct AwesomeName {
    pub awesome_name: String,
    pub language: NamedApiResource<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonSpecies {
    pub id: i64,
    pub name: String,
    pub order: i64,
    pub gender_rate: i64,
    pub capture_rate: i64,
    pub base_happiness: Option<i64>,
    pub is_baby: bool,
    pub is_legendary: bool,
    pub is_mythical: bool,
    pub hatch_counter: Option<i64>,
    pub has_gender_differences: bool,
    pub forms_switchable: bool,
    pub growth_rate: NamedApiResource<GrowthRate>,
    pub pokedex_numbers: Vec<PokemonSpeciesDexEntry>,
    pub egg_groups: Vec<NamedApiResource<EggGroup>>,
    pub color: NamedApiResource<PokemonColor>,
    pub shape: Option<NamedApiResource<PokemonShape>>,
    pub evolves_from_species: Option<NamedApiResource<PokemonSpecies>>,
    pub evolution_chain: Option<ApiResource<EvolutionChain>>,
    pub habitat: Option<NamedApiResource<PokemonHabitat>>,
    pub generation: NamedApiResource<Generation>,
    pub names: Vec<Name>,
    pub pal_park_encounters: Vec<PalParkEncounterArea>,
    pub flavor_text_entries: Vec<FlavorText>,
    pub form_descriptions: Vec<Description>,
    pub genera: Vec<Genus>,
    pub varieties: Vec<PokemonSpeciesVariety>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Genus {
    pub genus: String,
    pub language: NamedApiResource<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonSpeciesDexEntry {
    pub entry_number: i64,
    pub pokedex: NamedApiResource<Pokedex>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PalParkEncounterArea {
    pub base_score: i64,
    pub rate: i64,
    pub area: NamedApiResource<PalParkArea>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonSpeciesVariety {
    pub is_default: bool,
    pub pokemon: NamedApiResource<Pokemon>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Stat {
    pub id: i64,
    pub name: String,
    pub game_index: i64,
    pub is_battle_only: bool,
    pub affecting_moves: MoveStatAffectSets,
    pub affecting_natures: NatureStatAffectSets,
    pub characteristics: Vec<ApiResource<Characteristic>>,
    pub move_damage_class: Option<NamedApiResource<MoveDamageClass>>,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveStatAffectSets {
    pub increase: Vec<MoveStatAffect>,
    pub decrease: Vec<MoveStatAffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveStatAffect {
    pub change: i64,
    #[serde(rename = "move")]
    pub move_: NamedApiResource<Move>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct NatureStatAffectSets {
    pub increase: Vec<NamedApiResource<Nature>>,
    pub decrease: Vec<NamedApiResource<Nature>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Type {
    pub id: i64,
    pub name: String,
    pub damage_relations: TypeRelations,
    pub past_damage_relations: Vec<TypeRelationsPast>,
    pub game_indices: Vec<GenerationGameIndex>,
    pub generation: NamedApiResource<Generation>,
    pub move_damage_class: Option<NamedApiResource<MoveDamageClass>>,
    pub names: Vec<Name>,
    pub pokemon: Vec<TypePokemon>,
    pub moves: Vec<NamedApiResource<Move>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct TypePokemon {
    pub slot: i64,
    pub pokemon: NamedApiResource<Pokemon>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct TypeRelations {
    pub no_damage_to: Vec<NamedApiResource<Type>>,
    pub half_damage_to: Vec<NamedApiResource<Type>>,
    pub double_damage_to: Vec<NamedApiResource<Type>>,
    pub no_damage_from: Vec<NamedApiResource<Type>>,
    pub half_damage_from: Vec<NamedApiResource<Type>>,
    pub double_damage_from: Vec<NamedApiResource<Type>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct TypeRelationsPast {
    pub generation: NamedApiResource<Generation>,
    pub damage_relations: TypeRelations,
}
