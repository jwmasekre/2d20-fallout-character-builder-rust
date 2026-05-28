use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::character::DamageType;

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    MainMenu,
    NewCharSetup,
    Settings,
    LoadCharacter,
    ImportCharacter,
    OriginSelect,
    SpecialAssignment,
    SkillAssignment,
    PerkSelect,
    StatCalculation,
    BackgroundSelect,
    CharacterReview,
    CharacterSheet,
}
pub struct AppConfig {
    pub theme_index: usize,
    pub db_path: PathBuf,
    pub font_path: Option<PathBuf>,
    pub font_size: f32,
    pub ui_scale: f32,
    pub crt_distortion: f32,
    pub crt_scanline_strength: f32,
    pub crt_vignette_multiplier: f32,
    pub crt_vignette_exponent: f32,
    pub crt_roll_speed: f32,
    pub crt_tint_strength: f32,
    pub crt_chromatic_aberration: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_index: 0,
            db_path: std::env::current_exe().unwrap().parent().unwrap().join("fallout_2d20.db"),
            font_path: None,
            font_size: 20.0,
            ui_scale: 1.0,
            crt_distortion: 0.04,
            crt_scanline_strength: 0.04,
            crt_vignette_multiplier: 16.0,
            crt_vignette_exponent: 0.15,
            crt_roll_speed: 0.08,
            crt_tint_strength: 0.05,
            crt_chromatic_aberration: 0.001,
        }
    }
}

impl AppConfig {
    pub fn set_ui_scale(&mut self) {
        self.ui_scale = self.font_size / 20.0;
    }
}

#[derive(PartialEq, Clone)]
pub enum BwLk {
    BlackWidow,
    LadyKiller,
}

impl BwLk {
    pub fn to_perk_string(&self) -> &str {
        match self {
            BwLk::BlackWidow => "Black Widow (masc)",
            BwLk::LadyKiller => "Lady Killer (fem)"
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum MmCf {
    MechanicalMenace,
    ClassFreak,
}

impl MmCf {
    pub fn to_perk_string(&self) -> &str {
        match self {
            MmCf::MechanicalMenace => "Mechanical Menace (robots)",
            MmCf::ClassFreak => "Class Freak (mut. humans)"
        }
    }
}

pub enum PerkResolution {
    IntenseTraining {
        selected_stat: Option<usize>
    },
    Skilled {
        skill_a: Option<usize>,
        skill_b: Option<usize>,
    },
    Tag {
        selected_skill: Option<usize>,
    },
    BwLk {
        version: Option<BwLk>,
    },
    MmCf {
        version: Option<MmCf>,
    },
}

pub struct PerkResolutionPopup {
    pub perk_id: i32,
    pub perk_name: String,
    pub resolution: PerkResolution,
    pub perk_add: bool,
    pub open: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PreRelease {
    None,
    Alpha,
    Beta,
    ReleaseCandidate,
}

impl PreRelease {
    pub fn as_str(&self) -> &'static str {
        match self {
            PreRelease::Alpha => "-alpha",
            PreRelease::Beta => "-beta",
            PreRelease::ReleaseCandidate => "-rc",
            PreRelease::None => "",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub prerelease: PreRelease,
    pub prerelease_ver: i32,
}

impl Version {
    pub fn as_string(&self) -> String {
        let is_prerelease = self.prerelease != PreRelease::None;
        if is_prerelease {
            return format!("{}.{}.{}{}.{}",self.major, self.minor, self.patch, self.prerelease.as_str(), self.prerelease_ver)
        }
        return format!("{}.{}.{}",self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone)]
pub struct WeaponEffect {
    pub name: String,
    pub value: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct WeaponQuality {
    pub name: String,
    pub value: Option<i32>,
}

#[derive(Debug, Clone)]
pub enum ModEffect {
    SetDamage(i32),           // "XCD Dam"
    AddDamage(i32),              // "+XCD Dam"
    SubDamage(i32),              // "-XCD Dam"
    SetRate(i32),                // "X Rate"
    AddRate(i32),                // "+X Rate"
    SubRate(i32),                // "-X Rate"
    AddRange(),               // "+X Range"
    SubRange(),               // "-X Range"
    Gain(String, Option<i32>),   // "Gain X" or "Gain X Y"
    Lose(String, Option<i32>),   // "Lose X" or "Lose X Y"
    AllowsMods(String),          // "Allows X Mods"
    AddWeapon(String),           // "Add X weapon"
    SetDamageType(DamageType),       // "Dam Type = X"
    SetAmmo(String),             // "Ammo = X"
    Unknown(String),             // fallback
}

pub struct ModEffectList {
    pub dam_set: i32,
    pub dam_add: i32,
    pub dam_sub: i32,
    pub rat_set: i32,
    pub rat_add: i32,
    pub rat_sub: i32,
    pub rng_add: i32,
    pub rng_sub: i32,
    pub e_gain: Vec<(String, Option<i32>)>,
    pub q_gain: Vec<(String, Option<i32>)>,
    pub e_lose: Vec<(String, Option<i32>)>,
    pub q_lose: Vec<(String, Option<i32>)>,
    pub mods: String,
    pub weap: String,
    pub dam_type: DamageType,
    pub ammo: Option<String>,
    pub unk: String,
}

impl ModEffectList {
    pub fn new() -> Self {
        Self {
            dam_set: 0,
            dam_add: 0,
            dam_sub: 0,
            rat_set: 0,
            rat_add: 0,
            rat_sub: 0,
            rng_add: 0,
            rng_sub: 0,
            e_gain: vec![],
            e_lose: vec![],
            q_gain: vec![],
            q_lose: vec![],
            mods: "".to_string(),
            weap: "".to_string(),
            dam_type: DamageType::None,
            ammo: Some("".to_string()),
            unk: "".to_string(),
        }
    }
}
