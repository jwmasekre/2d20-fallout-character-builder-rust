use serde::{Deserialize, Serialize};
//use std::collections::HashMap;
use crate::character::CharPerk;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RangeBand {
    #[serde(rename = "")]
    None,
    R,
    C,
    M,
    L,
    X,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DamageType {
    Physical,
    Energy,
    #[serde(rename = "Physical/Energy")]
    PhysicalEnergy,
    Radiation,
    #[serde(rename = "Energy/Radiation")]
    EnergyRadiation,
    Poison,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharAmmo {
    pub ammo: i16,
    pub ammo_name: String,
    pub quantity: String, // "number | string" in TS -> String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendaryInner {
    pub name: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Legendary {
    pub is_legendary: bool,
    pub legendary: Option<LegendaryInner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharWeaponModInstalled {
    pub mod_id: i16,
    pub mod_name: String,
    pub mod_effect: Vec<String>,
    pub mod_weight: i16,
    pub mod_cost: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharWeaponMod {
    pub available: bool,
    pub installed: Option<CharWeaponModInstalled>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharWeaponMods {
    pub receiver: CharWeaponMod,
    pub barrel: CharWeaponMod,
    pub stock: CharWeaponMod,
    pub grip: CharWeaponMod,
    pub magazine: CharWeaponMod,
    pub sights: CharWeaponMod,
    pub muzzle: CharWeaponMod,
    pub capacitors: CharWeaponMod,
    pub dish: CharWeaponMod,
    pub fuel: CharWeaponMod,
    pub tank: CharWeaponMod,
    pub nozzle: CharWeaponMod,
    pub blade: CharWeaponMod,
    pub blunt: CharWeaponMod,
    pub frame: CharWeaponMod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub id: i16,
    pub name: String,
    pub skill_id: i16,      // "type" in schema → skill row id
    pub dam: String,
    pub dtype: DamageType,      // or DamageType enum later
    pub rate: i16,
    pub range: char,
    pub wgt: i16,
    pub cost: i16,
    pub rarity: i16,
    pub ammo_id: i16,
    pub sourcebook_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEffect {
    pub id: i16,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponEffectRow {
    pub id: i16,
    pub weapon_id: i16,
    pub effect_id: i16,
    pub effect_val: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quality {
    pub id: i16,
    pub name: String,
    pub description: String,
    pub opposed_to: Option<i16>, // self-FK to qualities.id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponQualityRow {
    pub id: i16,
    pub weapon_id: i16,
    pub qual_id: i16,
    pub qual_val: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponSlot {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponMod {
    pub id: i16,
    pub name: String,
    pub prefix: String,
    pub effects: Vec<String>,
    pub slot_id: i16,   // FK to weapon_slots.id
    pub wgt: i16,
    pub cost: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponModAvailable {
    pub id: i16,
    pub weapon_id: i16,
    pub mod_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponSlotAvailable {
    pub id: i16,
    pub weapon_id: i16,
    pub slot_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponLegendary {
    pub id: i16,
    pub roll_table: i16,
    pub name: String,
    pub eff: String,
    pub sg_roll: i16,
    pub ew_roll: i16,
    pub bg_roll: i16,
    pub mw_roll: i16,
    pub u_roll: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ammo {
    pub id: i16,
    pub name: String,
    pub roll_quantity: String,
    pub wgt: i16,
    pub cost: i16,
    pub rarity: i16,
    pub sourcebook_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Apparel {
    pub id: i16,
    pub name: String,
    pub type_id: i16,     // FK to apparel_types
    pub dog: bool,
    pub phys_dr: i16,
    pub enrg_dr: i16,
    pub rads_dr: i16,
    pub eff: Vec<String>,
    pub wgt: i16,
    pub cost: i16,
    pub rarity: i16,
    pub base_health: i16,
    pub sourcebook_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApparelType {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyLocation {
    pub id: i16,
    pub name: String,
    pub alternate_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApparelCoverRow {
    pub id: i16,
    pub apparel_id: i16,
    pub location_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApparelSlot {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApparelMod {
    pub id: i16,
    pub name: String,
    pub slot_id: i16,      // FK -> apparel_slots.id
    pub phys_dr: i16,
    pub enrg_dr: i16,
    pub rads_dr: i16,
    pub health: i16,
    pub effects: Vec<String>,
    pub wgt: i16,
    pub cost: i16,
    pub skill_id: i16,     // FK -> skills.id
}

/// apparel_mod_available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApparelModAvailable {
    pub id: i16,
    pub apparel_id: i16,
    pub mod_id: i16,
}

/// apparel_slot_available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApparelSlotAvailable {
    pub id: i16,
    pub apparel_id: i16,
    pub slot_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorLegendary {
    pub id: i16,
    pub name: String,
    pub eff: String,
    pub head_roll: i16,
    pub arm_roll: i16,
    pub torso_roll: i16,
    pub leg_roll: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerArmorPiece {
    /// FK into apparel.id
    pub apparel: crate::Apparel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerArmorMod {
    /// FK into apparel_mods.id
    pub base: crate::ApparelMod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotModule {
    pub id: i16,
    pub name: String,
    pub eff: Vec<String>,
    pub wgt: i16,
    pub cost: i16,
    pub rarity: i16,
    pub sourcebook_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotModulePerkRow {
    pub id: i16,
    pub robot_module_id: i16,   // FK -> robot_modules.id
    pub perk_id: i16,           // FK -> perks.id
    pub rank: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotModulePerk {
    pub perk: CharPerk,
    pub rank: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableType {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumable {
    pub id: i16,
    pub name: String,
    pub r#type: i16,     // FK to consumable_types
    pub heals: i16,
    pub eff: Vec<String>,
    pub rads: i16,
    pub wgt: i16,
    pub cost: i16,
    pub rarity: i16,
    pub duration: char,
    pub addiction: String,
    pub sourcebook_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gear {
    pub id: i16,
    pub name: String,
    pub eff: Vec<String>,
    pub wgt: i16,
    pub cost: i16,
    pub rarity: i16,
    pub sourcebook_id: i16,
}