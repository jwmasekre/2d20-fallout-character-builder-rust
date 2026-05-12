use crate::{AppScreen, character::{AmmoData, AmmoInv, Apparel, ApparelType, Background, BodyLocation, Character, Consumable, ConsumableType, DamageType, Gear, Junk, RobotModule, Skill, Weapon, WeaponMods, WeaponSlot}, db::Db, screens::{character_review::ReviewState, origin_select::OriginState, perk_select::PerkState, skill_assignment::SkillState, special_assignment::SpecialState}, theme::render_window};
use std::collections::{HashMap, HashSet};
use imgui::Ui;
use sdl2::video::Window;
use sqlx::SqlitePool;
//use rand::rng;

//db structs
#[derive(Debug, Clone)]
pub struct BackgroundRow {
    pub id: i32,
    pub origin_id: i32,
    pub name: String,
    pub desc: String,
    pub caps: i32,
    pub misc: String,
    pub trinket: i32,
    pub food: i32,
    pub forage: i32,
    pub bev: i32,
    pub chem: i32,
    pub ammo: i32,
    pub aid: i32,
    pub odd: i32,
    pub outcast: i32,
    pub junk: i32,
}

#[derive(Debug, Clone)]
pub struct WeaponRow {
    pub id: i32,
    pub _bg_id: i32,
    pub weapon_id: i32,
    pub weapon_name: String,
    pub mod_id: Option<i32>,
    pub mod_name: Option<String>,
    pub alt_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ApparelRow {
    pub id: i32,
    pub _bg_id: i32,
    pub apparel_id: i32,
    pub apparel_name: String,
    pub alt_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ConsumableRow {
    pub id: i32,
    pub _bg_id: i32,
    pub consumable_id: i32,
    pub consumable_name: String,
    pub alt_id: Option<i32>,
    pub wgt: i32,
}

#[derive(Debug, Clone)]
pub struct RobotModuleRow {
    pub id: i32,
    pub _bg_id: i32,
    pub module_id: i32,
    pub module_name: String,
    pub alt_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct AmmoRow {
    pub ammo_id: i32,
    pub ammo_name: String,
    pub quantity: String,
    pub bg_weapon_id: i32,
}

#[derive(Debug, Clone)]
pub struct GearRow {
    pub gear_id: i32,
    pub gear_name: String,
    pub wgt: i32,
}

//loading all the background data from the db
pub fn load_backgrounds(db: &Db) -> Vec<BackgroundRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"SELECT id, origin_id, name, description, caps, misc, trinket, food, forage, bev, chem, ammo, aid, odd, outcast, junk
               FROM backgrounds ORDER BY id"#
        )
        .fetch_all(&db.pool).await
    });
    match result {
        Ok(rows) => rows.into_iter().map(|r| BackgroundRow {
            id: r.id as i32,
            name: r.name.unwrap_or_default(),
            origin_id: r.origin_id.unwrap_or_default() as i32,
            desc: r.description.unwrap_or_default(),
            caps: r.caps.unwrap_or_default() as i32,
            misc: r.misc.unwrap_or_default(),
            trinket: r.trinket.unwrap_or_default() as i32,
            food: r.food.unwrap_or_default() as i32,
            forage: r.forage.unwrap_or_default() as i32,
            bev: r.bev.unwrap_or_default() as i32,
            chem: r.chem.unwrap_or_default() as i32,
            ammo: r.ammo.unwrap_or_default() as i32,
            aid: r.aid.unwrap_or_default() as i32,
            odd: r.odd.unwrap_or_default() as i32,
            outcast: r.outcast.unwrap_or_default() as i32,
            junk: r.junk.unwrap_or_default() as i32,
        }).collect(),
        Err(e) => { eprintln!("load_backgrounds: {e}"); vec![] }
    }
}
pub fn load_background_equipment(db: &Db, id: i32) -> ResolvedBackground {
    let background = load_backgrounds(db).into_iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| BackgroundRow {
            id,
            name: String::new(),
            desc: String::new(),
            origin_id: 0,
            caps: 0,
            misc: String::new(),
            trinket: 0,
            food: 0,
            forage: 0,
            bev: 0,
            chem: 0,
            ammo: 0,
            aid: 0,
            odd: 0,
            outcast: 0,
            junk: 0,
        });
    //weapons
    let weapon_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bw.id, bw.background_id, bw.weapon_id, bw.mod_id, bw.alt_id, w.name AS weapon_name, wm.name AS mod_name
               FROM background_weapons bw
               JOIN weapons w ON w.id = bw.weapon_id
               LEFT JOIN weapon_mods wm ON wm.id = bw.mod_id
               WHERE bw.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let weapons: Vec<WeaponRow> = weapon_rows.into_iter().map(|r| WeaponRow {
        id: r.id as i32,
        _bg_id: r.id as i32,
        weapon_id: r.weapon_id.unwrap_or_default() as i32,
        weapon_name: r.weapon_name.unwrap_or_default(),
        mod_id: r.mod_id.map(|i| i as i32),
        mod_name: r.mod_name,
        alt_id: r.alt_id.map(|i| i as i32),
    }).collect();
    //ammo
    let ammo_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT ba.ammo_id, ba.quantity, ba.bg_weapon_id, a.name AS ammo_name
               FROM background_ammo ba
               JOIN ammo a ON a.id = ba.ammo_id
               WHERE ba.bg_weapon_id IN (
                   SELECT id FROM background_weapons WHERE background_id = ?
               )"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let ammo: Vec<AmmoRow> = ammo_rows.into_iter().map(|r| AmmoRow {
        ammo_id: r.ammo_id.unwrap_or_default() as i32,
        ammo_name: r.ammo_name.unwrap_or_default(),
        quantity: r.quantity.unwrap_or_default(),
        bg_weapon_id: r.bg_weapon_id.unwrap_or_default() as i32,
    }).collect();
    //apparel
    let apparel_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT ba.id, ba.background_id, ba.apparel_id, ba.alt_id,
                      a.name AS apparel_name
               FROM background_apparel ba
               JOIN apparel a ON a.id = ba.apparel_id
               WHERE ba.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let apparel: Vec<ApparelRow> = apparel_rows.into_iter().map(|r| ApparelRow {
        id: r.id as i32,
        _bg_id: r.background_id.unwrap_or_default() as i32,
        apparel_id: r.apparel_id.unwrap_or_default() as i32,
        apparel_name: r.apparel_name.unwrap_or_default(),
        alt_id: r.alt_id.map(|i| i as i32),
    }).collect();
    //consumables
    let consumable_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bc.id, bc.background_id, bc.consumable_id, bc.alt_id, c.wgt, c.name AS consumable_name
               FROM background_consumables bc
               JOIN consumables c ON c.id = bc.consumable_id
               WHERE bc.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let consumables: Vec<ConsumableRow> = consumable_rows.into_iter().map(|r| ConsumableRow {
        id: r.id as i32,
        _bg_id: r.background_id.unwrap_or_default() as i32,
        consumable_id: r.consumable_id.unwrap_or_default() as i32,
        consumable_name: r.consumable_name.unwrap_or_default(),
        alt_id: r.alt_id.map(|i| i as i32),
        wgt: r.wgt.unwrap_or_default() as i32,
    }).collect();
    //robot mods
    let module_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT brm.id, brm.background_id, brm.robot_module_id, brm.alt_id, rm.name AS module_name
               FROM background_robot_modules brm
               JOIN robot_modules rm ON rm.id = brm.robot_module_id
               WHERE brm.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let robot_modules: Vec<RobotModuleRow> = module_rows.into_iter().map(|r| RobotModuleRow {
        id: r.id as i32,
        _bg_id: r.background_id.unwrap_or_default() as i32,
        module_id: r.robot_module_id.unwrap_or_default() as i32,
        module_name: r.module_name.unwrap_or_default(),
        alt_id: r.alt_id.map(|i| i as i32),
    }).collect();
    //gear - no choices for gear
    let gear_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bg.gear_id, g.name AS gear_name, g.wgt
               FROM background_gear bg
               JOIN gear g ON g.id = bg.gear_id
               WHERE bg.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let gear: Vec<GearRow> = gear_rows.into_iter().map(|r| GearRow {
        gear_id: r.gear_id.unwrap_or_default() as i32,
        gear_name: r.gear_name.unwrap_or_default(),
        wgt: r.wgt.unwrap_or_default() as i32,
    }).collect();
    //include the miscellaneous stuff
    let misc = serde_json::from_str::<Vec<String>>(&background.misc)
        .unwrap_or_default()
        .join(", ");

    //put them all together
    ResolvedBackground {
        id: background.id,
        name: background.name,
        desc: background.desc,
        weapon_slots: resolve_weapon_slots(weapons),
        apparel_slots: resolve_apparel_slots(apparel),
        consumable_slots: resolve_consumable_slots(consumables),
        robot_module_slots: resolve_robot_module_slots(robot_modules),
        ammo,
        gear,
        caps: background.caps,
        misc,
        trinket: background.trinket,
        food: background.food,
        forage: background.forage,
        bev: background.bev,
        chem: background.chem,
        ammo_count: background.ammo,
        aid: background.aid,
        odd: background.odd,
        outcast: background.outcast,
        junk: background.junk,
    }
}


//used to handle applying mod effects properly
pub struct EffectNameSets {
    pub effect_names: HashSet<String>,
    pub quality_names: HashSet<String>,
}

impl EffectNameSets {
    pub fn load(db: &Db) -> Self {
        let effects = db.block_on(async {
            sqlx::query!("SELECT name FROM dam_effects").fetch_all(&db.pool).await
        }).unwrap_or_default();
        let qualities = db.block_on(async {
            sqlx::query!("SELECT name FROM qualities").fetch_all(&db.pool).await
        }).unwrap_or_default();
        //this needs to be adjusted to handle X effects/qualities
        Self {
            effect_names: effects.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
            quality_names: qualities.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
        }
    }
    pub async fn load_async(pool: &SqlitePool) -> Self {
        let effects = sqlx::query!("SELECT name FROM dam_effects").fetch_all(pool).await.unwrap_or_default();
        let qualities = sqlx::query!("SELECT name FROM qualities").fetch_all(pool).await.unwrap_or_default();
        //this needs to be adjusted to handle X effects/qualities
        Self {
            effect_names: effects.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
            quality_names: qualities.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
        }
    }
    pub fn qual_not_eff(&self, name: &str) -> Option<bool> {
        let mut res = self.quality_names.contains(&name.to_lowercase());
        if res { return Some(res) } else {
            res = self.effect_names.contains(&name.to_lowercase());
            if res { return Some(!res) } else { None }
        }
    }
}

#[derive(Debug)]
pub struct BackgroundState {
    pub all_backgrounds: Vec<BackgroundRow>,
    pub selected_index: Option<usize>,
    pub current_background: Option<ResolvedBackground>,
    pub weapon_selections: Vec<SlotSelection>,
    pub apparel_selections: Vec<SlotSelection>,
    pub consumable_selections: Vec<SlotSelection>,
    pub robot_module_selections: Vec<SlotSelection>,
    pub equipment_changed: bool,
}

impl BackgroundState {
    pub fn new(db: &Db) -> Self {
        Self {
            all_backgrounds: load_backgrounds(db),
            selected_index: None,
            current_background: None,
            weapon_selections: vec![],
            apparel_selections: vec![],
            consumable_selections: vec![],
            robot_module_selections: vec![],
            equipment_changed: false,
        }
    }
    pub fn reset(&mut self) {
        self.selected_index = None;
        self.current_background = None;
        self.weapon_selections = vec![];
        self.apparel_selections = vec![];
        self.consumable_selections = vec![];
        self.robot_module_selections = vec![];
        self.equipment_changed = false;
    }
    fn origin_backgrounds(&self, character: Character) -> Vec<(usize, &BackgroundRow)> {
        self.all_backgrounds.iter()
            .enumerate()
            .filter(|(_, bg)| {
                character.origin
                    .clone()
                    .map(|o| bg.origin_id == o.id)
                    .unwrap_or(true)
            })
            .collect()
    }
    pub fn reset_selection(&mut self) {
        self.selected_index = None;
        self.current_background = None;
        self.weapon_selections.clear();
        self.apparel_selections.clear();
        self.consumable_selections.clear();
        self.robot_module_selections.clear();
        self.equipment_changed = true;
    }
    fn load_background(&mut self, db: &Db, index: usize) {
        let bg_id = self.all_backgrounds[index].id;
        self.selected_index = Some(index);
        let background = load_background_equipment(db, bg_id);
        self.weapon_selections = default_selections(&background.weapon_slots);
        self.apparel_selections = default_apparel_selections(&background.apparel_slots);
        self.consumable_selections = default_selections(&background.consumable_slots);
        self.robot_module_selections = default_selections(&background.robot_module_slots);
        self.current_background = Some(background);
    }
    pub fn is_complete(&mut self, equipment: &mut EquipmentState, db: &Db, character: &Character, review: &mut ReviewState) -> bool {
        let complete = self.selected_index.is_some() && selections_complete(&self.weapon_selections) && selections_complete(&self.apparel_selections) && selections_complete(&self.consumable_selections) && selections_complete(&self.robot_module_selections);
        if complete && self.equipment_changed {
            equipment.load(db, self, character);
            self.equipment_changed = false;
            review.loaded = false;
        }
        complete
    }
}

//using this to basically handle the inventory so we can pass it over to review
//review will apply the inventory on acceptance
#[derive(Debug)]
pub struct EquipmentState {
    pub weapons: Vec<Weapon>,
    pub ammo: Vec<AmmoInv>,
    pub apparel: Vec<Apparel>,
    pub robot_modules: Vec<RobotModule>,
    pub consumables: Vec<Consumable>,
    pub gear: Vec<Gear>,
    pub junk: Junk,
    pub misc: Vec<String>,
}
impl EquipmentState {
    pub fn new() -> Self {
        Self {
            weapons: vec![],
            ammo: vec![],
            apparel: vec![],
            robot_modules: vec![],
            consumables: vec![],
            gear: vec![],
            junk: Junk {
                common: 0,
                uncommon: 0,
                rare: 0,
            },
            misc: vec![],
        }
    }
    pub fn reset(&mut self) {
        self.weapons = vec![];
        self.ammo = vec![];
        self.apparel = vec![];
        self.robot_modules = vec![];
        self.consumables = vec![];
        self.gear = vec![];
        self.junk = Junk {
            common: 0,
            uncommon: 0,
            rare: 0,
        };
        self.misc = vec![];
    }
    pub fn load(&mut self, db: &Db, state: &BackgroundState, character: &Character) {
        if state.current_background.is_some() {
            (self.weapons, self.ammo) = resolve_weapons(db, &state.current_background.clone().unwrap(), &state.weapon_selections, character);
            self.apparel = resolve_apparel(db, &state.current_background.clone().unwrap(), &state.apparel_selections);
            self.consumables = resolve_consumables(db, &state.current_background.clone().unwrap(), &state.consumable_selections);
            self.robot_modules = resolve_robot_modules(db, &state.current_background.clone().unwrap(), &state.robot_module_selections);
            (self.gear, self.junk, self.misc) = resolve_remaining_eq(db, &state.current_background.clone().unwrap());
        }
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

pub fn parse_damage_type(s: &str) -> DamageType {
    match s {
        "Ph"    => DamageType::Ph,
        "En"    => DamageType::En,
        "Ph/En" => DamageType::PhEn,
        "Rad"   => DamageType::Rad,
        "En/Rad"=> DamageType::EnRad,
        "Poi"   => DamageType::Poi,
        "All"   => DamageType::All,
        _       => DamageType::None,
    }
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
    fn new() -> Self {
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

//need to review
pub fn parse_mod_effect(s: &str) -> ModEffect {
    let s = s.trim();
    // Dam Type = X
    if let Some(rest) = s.strip_prefix("Dam Type =").or_else(|| s.strip_prefix("Dam Type=")) {
        let val = rest.trim();
        let dtype = if val.eq_ignore_ascii_case("Both Physical and Energy") {
            parse_damage_type("Ph/En")
        } else {
            parse_damage_type(val)
        };
        return ModEffect::SetDamageType(dtype);
    }
    // Ammo = X
    if let Some(rest) = s.strip_prefix("Ammo =").or_else(|| s.strip_prefix("Ammo=")) {
        return ModEffect::SetAmmo(rest.trim().to_string());
    }
    // Allows X Mods
    if let Some(rest) = s.strip_prefix("Allows ") {
        if let Some(mod_name) = rest.strip_suffix(" Mods") {
            return ModEffect::AllowsMods(mod_name.trim().to_string());
        }
    }
    // Add X weapon
    if let Some(rest) = s.strip_prefix("Add ") {
        if let Some(weap) = rest.strip_suffix(" weapon") {
            return ModEffect::AddWeapon(weap.trim().to_string());
        }
    }
    // Gain X / Gain X Y / Gain X(Y) / Gain X (Y)
    if let Some(rest) = s.strip_prefix("Gain ") {
        let (name, val) = parse_name_and_value(rest);
        return ModEffect::Gain(name, val);
    }
    // Lose X / Lose X Y
    if let Some(rest) = s.strip_prefix("Lose ") {
        let (name, val) = parse_name_and_value(rest);
        return ModEffect::Lose(name, val);
    }
    // +X Range / -X Range
    if let Some(rest) = s.strip_suffix(" Range") {
        if let Some(_) = rest.strip_prefix('+').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::AddRange();
        }
        if let Some(_) = rest.strip_prefix('-').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::SubRange();
        }
    }
    // +X Rate / -X Rate / X Rate
    if let Some(rest) = s.strip_suffix(" Rate") {
        if let Some(n) = rest.strip_prefix('+').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::AddRate(n);
        }
        if let Some(n) = rest.strip_prefix('-').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::SubRate(n);
        }
        if let Ok(n) = rest.parse::<i32>() {
            return ModEffect::SetRate(n);
        }
    }
    // XCD Dam / +XCD Dam / -XCD Dam
    if let Some(rest) = s.strip_suffix(" Dam") {
        if let Some(inner) = rest.strip_prefix('+') {
            // "+3CD Dam" → extract just the number
            let n: i32 = inner.trim_end_matches(|c: char| c.is_alphabetic())
                .parse().unwrap_or(0);
            return ModEffect::AddDamage(n);
        }
        if let Some(inner) = rest.strip_prefix('-') {
            let n: i32 = inner.trim_end_matches(|c: char| c.is_alphabetic())
                .parse().unwrap_or(0);
            return ModEffect::SubDamage(n);
        }
        // "3CD Dam" or "2CD Dam" — full replacement
        return ModEffect::SetDamage(rest.trim_end_matches(|c: char| c.is_alphabetic()).parse().unwrap_or(0));
    }
    ModEffect::Unknown(s.to_string())
}

/// Parse "Name" or "Name 3" or "Name(3)" or "Name (3)"
fn parse_name_and_value(s: &str) -> (String, Option<i32>) {
    // Try "Name(X)" or "Name (X)"
    if let Some(paren_start) = s.rfind('(') {
        let name = s[..paren_start].trim().to_string();
        let val_str = s[paren_start+1..].trim_end_matches(')').trim();
        if let Ok(v) = val_str.parse::<i32>() {
            return (name, Some(v));
        }
    }
    // Try "Name X" where last token is a number
    if let Some(last_space) = s.rfind(' ') {
        let maybe_num = &s[last_space+1..];
        if let Ok(v) = maybe_num.parse::<i32>() {
            return (s[..last_space].trim().to_string(), Some(v));
        }
    }
    (s.trim().to_string(), None)
}

fn apply_gain(
    effects: &mut Vec<WeaponEffect>,
    qualities: &mut Vec<WeaponQuality>,
    name: &str,
    val: Option<i32>,
    names: &EffectNameSets,
    mod_eff: &mut ModEffectList,
) {
    if names.qual_not_eff(name).is_some() {
        if names.qual_not_eff(name).unwrap() {
            if let Some(existing) = qualities.iter_mut().find(|q| q.name.eq_ignore_ascii_case(name)) {
                match (existing.value, val) {
                    (Some(qv), Some(v)) => existing.value = Some(qv + v),
                    (None, Some(v))     => existing.value = Some(v),
                    _                   => {}
                }
            } else {
                qualities.push(WeaponQuality { name: name.to_string(), value: val });
            }
            mod_eff.q_gain.push((name.to_string(), val));
        } else {
            if let Some(existing) = effects.iter_mut().find(|e| e.name.eq_ignore_ascii_case(name)) {
                match (existing.value, val) {
                    (Some(ev), Some(v)) => existing.value = Some(ev + v),
                    (None, Some(v))     => existing.value = Some(v),
                    _                   => {}
                }
            } else {
                effects.push(WeaponEffect { name: name.to_string(), value: val });
            }
            mod_eff.e_gain.push((name.to_string(), val));
        }
    } else {
        eprintln!("[apply_gain] unknown name '{}' — not in dam_effects or qualities", name);
    }
}

fn apply_lose(
    effects: &mut Vec<WeaponEffect>,
    qualities: &mut Vec<WeaponQuality>,
    name: &str,
    val: Option<i32>,
    names: &EffectNameSets,
    mod_eff: &mut ModEffectList,
) {
    if names.qual_not_eff(name).is_some() {
        if names.qual_not_eff(name).unwrap() {
            if let Some(pos) = qualities.iter().position(|q| q.name.eq_ignore_ascii_case(name)) {
                match (qualities[pos].value, val) {
                    (Some(qv), Some(v)) if v >= qv => { qualities.remove(pos); }
                    (Some(qv), Some(v))             => { qualities[pos].value = Some(qv - v); }
                    _                               => { qualities.remove(pos); }
                }
            }
            mod_eff.q_lose.push((name.to_string(),val))
        } else {
            if let Some(pos) = effects.iter().position(|e| e.name.eq_ignore_ascii_case(name)) {
                match (effects[pos].value, val) {
                    (Some(ev), Some(v)) if v >= ev => { effects.remove(pos); }
                    (Some(ev), Some(v))             => { effects[pos].value = Some(ev - v); }
                    _                               => { effects.remove(pos); }
                }
            }
            mod_eff.e_lose.push((name.to_string(),val))
        }
    } else {
        eprintln!("[apply_lose] unknown name '{}' — not in dam_effects or qualities", name);
    }
}

fn resolve_weapons(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
    character: &Character
) -> (Vec<Weapon>,Vec<AmmoInv>) {
    //grab all the weapon ids that were selected
    let selected_weapon_ids: Vec<i32> = background.weapon_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (WeaponSelSlot::Fixed(opt), _) => vec![opt.bg_weapon_id],
            (WeaponSelSlot::Choice(opts), SlotSelection::Chosen(i)) if *i < opts.len() => vec![opts[*i].bg_weapon_id],
            (WeaponSelSlot::ManyForOne(give_up,get_one ), SlotSelection::ManyForOneChosen(choice)) => if *choice == 0 {
                vec![get_one.bg_weapon_id]
            } else {
                give_up.iter().map(|w| w.bg_weapon_id).collect()
            },
            _ => vec![],
        })
        .collect();
    //if nothing is selected, send an empty vector
    if selected_weapon_ids.is_empty() { return (vec![],vec![]) }

    //grab the entire weapon's data for each selected weapon from the db
    let id_json = serde_json::to_string(&selected_weapon_ids).unwrap_or_default();
    let rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT
                bw.id        AS bg_weapon_id,
                w.id         AS weapon_id,
                w.name       AS weapon_name,
                w.dam, w.dtype, w.rate, w.range, w.wgt,
                s.name       AS skill_name,
                a.name       AS ammo_name,
                a.wgt        AS ammo_wgt,
                a.id         AS ammo_id,
                ba.quantity  AS ammo_quantity,
                wm.id        AS mod_id,
                wm.name      AS mod_name,
                wm.prefix    AS mod_prefix,
                wm.effects   AS mod_effects,
                wm.wgt       AS mod_wgt,
                wm.slot      AS mod_slot
            FROM background_weapons bw
            JOIN weapons w   ON w.id  = bw.weapon_id
            JOIN skills  s   ON s.id  = w.type
            LEFT JOIN weapon_mods wm ON wm.id = bw.mod_id
            LEFT JOIN background_ammo ba ON ba.bg_weapon_id = bw.id
            LEFT JOIN ammo a ON a.id = ba.ammo_id
            WHERE bw.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();
    let mut w_result: Vec<Weapon> = vec![];
    let mut a_result: Vec<AmmoInv> = vec![];

    for row in &rows {
        let weapon_id = row.weapon_id;
        //grab qualities
        let qual_rows = db.block_on(async {
            sqlx::query!(
                r#"SELECT q.name, wq.qual_val
                   FROM weapon_quals wq
                   JOIN qualities q ON q.id = wq.qual_id
                   WHERE wq.weapon_id = ?"#,
                weapon_id
            ).fetch_all(&db.pool).await
        }).unwrap_or_default();
        let mut qualities: Vec<WeaponQuality> = qual_rows.iter().map(|q| WeaponQuality {
            name: q.name.clone().unwrap_or_default(),
            value: q.qual_val.map(|v| v as i32),
        }).collect();

        //grab effects
        let eff_rows = db.block_on(async {
            sqlx::query!(
                r#"SELECT de.name, we.effect_val
                   FROM weapon_effects we
                   JOIN dam_effects de ON de.id = we.effect_id
                   WHERE we.weapon_id = ?"#,
                weapon_id
            ).fetch_all(&db.pool).await
        }).unwrap_or_default();
        let mut effects: Vec<WeaponEffect> = eff_rows.iter().map(|e| WeaponEffect {
            name: e.name.clone().unwrap_or_default(),
            value: e.effect_val.map(|v| v as i32),
        }).collect();

        let damage_str = row.dam.clone().unwrap_or_default();
        let mut damage: i32 = damage_str.trim_end_matches(|c: char| c.is_alphabetic()).parse().unwrap_or(0);
        let mut rate = row.rate.unwrap_or_default() as i32;
        let mut range = row.range.clone().unwrap_or_default();
        let damage_type_str = row.dtype.clone().unwrap_or("".to_string());
        let mut dam_type = parse_damage_type(&damage_type_str);
        let base_wgt = row.wgt.unwrap_or_default() as i32;
        let mod_wgt = row.mod_wgt.unwrap_or_default() as i32;
        let weight = base_wgt + mod_wgt;
        let name = row.weapon_name.clone().unwrap_or_default();
        let prefix = row.mod_prefix.clone().unwrap_or_default();
        let ammo_name = row.ammo_name.clone().unwrap_or("".to_string());

        //target number calcs
        let skill_name = row.skill_name.clone().unwrap_or_default();
        let special: Vec<i32> = character.special.special_block().iter().map(|s| s.value.clone()).collect();
        let skills: Vec<i32>  = character.skills.skill_block().iter().map(|s| s.total.clone()).collect();
        let tags: Vec<bool> = character.skills.skill_block().iter().map(|s| s.is_tagged()).collect();
        let (spec_index, skill_index, skill) = match skill_name.as_str() {
            "Melee Weapons" => (0,7,Skill::MeleeWeapons),
            "Unarmed" => (0,16,Skill::Unarmed),
            "Small Guns" => (5,11,Skill::SmallGuns),
            "Throwing" => (5,15,Skill::Throwing),
            "Energy Weapons" => (1,3,Skill::EnergyWeapons),
            "Explosives" => (1,4,Skill::Explosives),
            "Big Guns" => (2,2,Skill::BigGuns),
            _  => (6,0,Skill::Athletics),
        };
        let spec_value = special[spec_index];
        let skill_total = skills[skill_index];
        let tag = tags[skill_index];
        let target = skill_total + spec_value;

        let name_set = load_mod_effect(db);
        let weapon_mod_eff = resolve_mod_effect(name_set,row.mod_effects.clone(), &mut damage, &mut rate, &mut range, &mut effects, &mut qualities, &mut dam_type);

        let mut weapon_mods: Vec<WeaponMods> = vec![];
        weapon_mods.push(WeaponMods {
            slot: resolve_weapon_slot(row.mod_slot.unwrap_or(0)),
            installed: true,
            id: row.mod_id.unwrap_or(0) as i32,
            name: row.mod_name.clone().unwrap_or("".to_string()),
            prefix: row.mod_prefix.clone().unwrap_or("".to_string()),
            wgt: row.mod_wgt.unwrap_or(0) as i32,
            damage_set: weapon_mod_eff.dam_set,
            damage_chg: weapon_mod_eff.dam_add - weapon_mod_eff.dam_sub,
            rate_set: weapon_mod_eff.rat_set,
            rate_chg: weapon_mod_eff.rat_add - weapon_mod_eff.rat_sub,
            ammo_set: weapon_mod_eff.ammo,
            range_chg: weapon_mod_eff.rng_add - weapon_mod_eff.rng_sub,
            effect_add: weapon_mod_eff.e_gain,
            effect_rem: weapon_mod_eff.e_lose,
            quality_add: weapon_mod_eff.q_gain,
            quality_rem: weapon_mod_eff.q_lose,
            slot_add: weapon_mod_eff.mods,
            damage_type_set: Some(weapon_mod_eff.dam_type),
            weapon_add: weapon_mod_eff.weap,
            special_ability: weapon_mod_eff.unk,
        });

        let weap_eff_str: Vec<String> = effects.iter().map(|e| if e.value != Some(0) && e.value.is_some() { format!("{} {}", e.name, e.value.unwrap()) } else { e.name.clone() }).collect();
        let weap_qual_str: Vec<String> = qualities.iter().map(|q| if q.value != Some(0) && q.value.is_some() { format!("{} {}", q.name, q.value.unwrap()) } else { q.name.clone() }).collect();

        w_result.push(Weapon {
            id: weapon_id.unwrap_or(0) as i32,
            name,
            prefix,
            skill,
            target,
            tag,
            damage,
            effects: weap_eff_str,
            dam_type,
            rate,
            range,
            qualities: weap_qual_str,
            ammo: ammo_name.clone(),
            wgt: weight,
            mods: weapon_mods,
        });
        a_result.push(AmmoInv {
            ammo: AmmoData {
                id: row.ammo_id.unwrap_or(0) as i32,
                name: ammo_name.clone(),
                wgt: row.ammo_wgt.unwrap_or(0) as i32,
            },
            quantity: roll_cd(&row.ammo_quantity.clone().unwrap_or("".to_string()))
        })
    }
    (w_result,a_result)
}

pub fn resolve_weapon_slot(slot: i64) -> WeaponSlot {
    match slot {
        1 => WeaponSlot::Receiver,
        2 => WeaponSlot::Barrel,
        3 => WeaponSlot::Stock,
        4 => WeaponSlot::Grip,
        5 => WeaponSlot::Magazine,
        6 => WeaponSlot::Sights,
        7 => WeaponSlot::Muzzle,
        8 => WeaponSlot::Capacitors,
        9 => WeaponSlot::Dish,
        10 => WeaponSlot::Fuel,
        11 => WeaponSlot::Tank,
        12 => WeaponSlot::Nozzle,
        13 => WeaponSlot::Blade,
        14 => WeaponSlot::Blunt,
        15 => WeaponSlot::Frame,
        _ => WeaponSlot::None,
    }
}

pub fn load_mod_effect(db: &Db) -> EffectNameSets {
    EffectNameSets::load(db)
}

pub async fn load_mod_effect_async(pool: &SqlitePool) -> EffectNameSets {
    EffectNameSets::load_async(pool).await
}
    
pub fn resolve_mod_effect(name_set: EffectNameSets, eff: Option<String>, damage: &mut i32, rate: &mut i32, range: &mut String, effects: &mut Vec<WeaponEffect>, qualities: &mut Vec<WeaponQuality>, dam_type: &mut DamageType) -> ModEffectList {
    let ranges = ["R","C","M","L","X"];

    let mut weapon_mod_eff = ModEffectList::new();
    if let Some(mod_fx_json) = &eff {
        if let Ok(fx_strings) = serde_json::from_str::<Vec<String>>(mod_fx_json) {
            for fx_str in &fx_strings {
                match parse_mod_effect(fx_str) {
                    ModEffect::SetDamage(d) => {
                        *damage = d;
                        weapon_mod_eff.dam_set = d;
                    }
                    ModEffect::AddDamage(n) => {
                        // Extract leading number from e.g. "3CD", add n, reformat
                        *damage += n;
                        weapon_mod_eff.dam_add = n;
                    }
                    ModEffect::SubDamage(n) => {
                        *damage -= n;
                        if *damage < 1 { *damage = 1 }
                        weapon_mod_eff.dam_sub = n;
                    }
                    ModEffect::SetRate(r) => {
                        *rate = r;
                        weapon_mod_eff.rat_set = r;
                    }
                    ModEffect::AddRate(r) => {
                        *rate += r;
                        weapon_mod_eff.rat_add = r;
                    }
                    ModEffect::SubRate(r) => {
                        *rate = (*rate - r).max(0);
                        weapon_mod_eff.rat_sub = r;
                    }
                    ModEffect::AddRange() => {
                        let range_num = ranges.iter().position(|&r| r == range.as_str()).unwrap();
                        if range_num > 0 || range_num < 4 {
                            *range = ranges[range_num + 1].to_string();
                        }
                        weapon_mod_eff.rng_add = 1;
                    }
                    ModEffect::SubRange() => {
                        let range_num = ranges.iter().position(|&r| r == range.as_str()).unwrap();
                        if range_num > 1 || range_num < 5 {
                            *range = ranges[range_num - 1].to_string();
                        }
                        weapon_mod_eff.rng_sub = 1;
                    }
                    ModEffect::Gain(name, val) => {
                        apply_gain(effects, qualities, &name, val, &name_set, &mut weapon_mod_eff);
                    }
                    ModEffect::Lose(name, val) => {
                        apply_lose(effects, qualities, &name, val, &name_set, &mut weapon_mod_eff);
                    }
                    ModEffect::SetDamageType(dt) => {
                        *dam_type = dt.clone();
                        weapon_mod_eff.dam_type = dt;
                    }
                    ModEffect::SetAmmo(name) => {
                        // Ammo swap handled at character sheet save time
                        weapon_mod_eff.ammo = Some(name);
                    }
                    ModEffect::AllowsMods(name) => {
                        // Structural changes — handled at save time
                        weapon_mod_eff.mods = name;
                    }
                    ModEffect::AddWeapon(name) => {
                        weapon_mod_eff.weap = name;
                    }
                    ModEffect::Unknown(s) => {
                        eprintln!("Unknown mod effect: {s}");
                        weapon_mod_eff.unk = s;
                    }
                }
            }
        }
    }
    weapon_mod_eff
}

fn resolve_apparel(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
) -> Vec<Apparel> {
    let mut result: Vec<Apparel> = vec![];
    let selected_apparel_ids: Vec<i32> = background.apparel_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (ApparelSelSlot::Fixed(opt), _) => vec![opt.bg_apparel_id],
            (ApparelSelSlot::Choice(opts), SlotSelection::Chosen(i)) => vec![opts[*i].bg_apparel_id],
            (ApparelSelSlot::SingleOrDouble(single, double_choices), SlotSelection::SingleOrDoubleChosen(take_single, double_picks)) => if *take_single {
                vec![single.bg_apparel_id]
            } else {
                vec![double_choices[0][double_picks[0].unwrap()].bg_apparel_id, double_choices[1][double_picks[1].unwrap()].bg_apparel_id]
            }
            (ApparelSelSlot::SingleOrPack(single, pack), SlotSelection::SingleOrPackChosen(choice)) => if *choice {
                vec![single.bg_apparel_id]
            } else {
                pack.iter().map(|a| a.bg_apparel_id).collect()
            },
            _ => vec![],
        })
        .collect();
    if selected_apparel_ids.is_empty() { return vec![] } 

    let id_json = serde_json::to_string(&selected_apparel_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                ba.id        AS bg_apparel_id,
                a.id         AS id,
                a.name       AS name,
                a.phys_dr    AS ph_dr,
                a.enrg_dr    AS en_dr,
                a.rads_dr    AS rd_dr,
                a.wgt        AS wgt,
                a.eff        AS effs,
                a.type       AS a_type
            FROM background_apparel ba
            JOIN apparel a   ON a.id  = ba.apparel_id
            WHERE ba.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        let apparel_id = row.id.unwrap() as i32;
        let cover_list: Vec<i64> = db.block_on(async {
            sqlx::query!(
                r#"SELECT
                    ac.location_id AS cid
                FROM apparel_covers ac
                WHERE ac.apparel_id = ?
                "#,
                apparel_id
            ).fetch_all(&db.pool).await
        }).unwrap_or_default().iter().map(|c| c.cid.unwrap()).collect();
        let covers = resolve_apparel_covers(cover_list);
        println!("{} resolved covers: {:?}", apparel_id, covers);
        let effects = vec![row.effs.unwrap_or("".to_string())];

        result.push(Apparel {
            id: apparel_id,
            name: row.name.clone().unwrap_or("".to_string()),
            prefix: "".to_string(),
            apparel_type: resolve_apparel_type(row.a_type.unwrap_or(0)),
            ph_dr: row.ph_dr.unwrap_or(0) as i32,
            en_dr: row.en_dr.unwrap_or(0) as i32,
            rd_dr: row.rd_dr.unwrap_or(0) as i32,
            wgt: row.wgt.unwrap_or(0) as i32,
            effects,
            covers,
            equipped: false,
        })
    }
    result
}

pub fn resolve_apparel_type(atype: i64) -> ApparelType {
    match atype {
        1 => ApparelType::Clothing,
        2 => ApparelType::Outfit,
        3 => ApparelType::Headgear,
        4 => ApparelType::Armor,
        5 => ApparelType::PowerArmor,
        6 => ApparelType::RobotArmor,
        _ => ApparelType::Clothing
    }
}

pub fn resolve_apparel_covers(results: Vec<i64>) -> Vec<BodyLocation> {
    let mut covers: Vec<BodyLocation> = vec![];
    println!("resolving covers: {:?}", results);
    for item in results {
        covers.push(match item {
            1 => BodyLocation::Head,
            2 => BodyLocation::ArmLeft,
            3 => BodyLocation::ArmRight,
            4 => BodyLocation::Torso,
            5 => BodyLocation::LegLeft,
            6 => BodyLocation::LegRight,
            7 => BodyLocation::Optics,
            8 => BodyLocation::Arm1,
            9 => BodyLocation::Arm2,
            10 => BodyLocation::Arm3,
            11 => BodyLocation::Body,
            12 => BodyLocation::Thruster,
            13 => BodyLocation::Wheel,
            _ => BodyLocation::None,
        })
    }
    covers
}

fn resolve_consumables(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
) -> Vec<Consumable> {
    let mut result: Vec<Consumable> = vec![];
    let selected_consumable_ids: Vec<i32> = background.consumable_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (ConsumableSelSlot::Fixed(opt), _) => vec![opt.bg_consumable_id],
            (ConsumableSelSlot::Choice(opts), SlotSelection::Chosen(i)) => vec![opts[*i].bg_consumable_id],
            (ConsumableSelSlot::ManyForOne(give_up,get_one ), SlotSelection::ManyForOneChosen(choice)) => if *choice == 0 {
                vec![get_one.bg_consumable_id]
            } else {
                give_up.iter().map(|c| c.bg_consumable_id).collect()
            },
            _ => vec![],
        })
        .collect();
    if selected_consumable_ids.is_empty() { return vec![] } 

    let id_json = serde_json::to_string(&selected_consumable_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                bc.id        AS bg_consumable_id,
                c.id         AS id,
                c.name       AS name,
                c.type       AS c_type, 
                c.heals      AS health,
                c.eff        AS effs,
                c.rads       AS rads,
                c.wgt        AS wgt,
                c.duration   AS duration,
                c.addiction  AS addiction
            FROM background_consumables bc
            JOIN consumables c ON c.id  = bc.consumable_id
            JOIN consumable_types ct ON ct.id  = c.type
            WHERE bc.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        if result.iter().any(|c| c.id == row.bg_consumable_id.unwrap_or(0) as i32) {
            let c_loc = result.iter().position(|c| c.id == row.bg_consumable_id.unwrap_or(0) as i32);
            result[c_loc.unwrap()].quantity += 1;
        } else {
            let addiction: i32 = row.addiction.unwrap_or(0) as i32;
            result.push(Consumable {
                id: row.id.unwrap_or(0) as i32,
                name: row.name.unwrap_or("".to_string()),
                consumable_type: resolve_consumable_type(row.c_type.unwrap_or(0)),
                health: row.health.unwrap_or(0) as i32,
                effects: vec![row.effs.unwrap_or("".to_string())],
                rads: row.rads.unwrap_or(0) as i32,
                wgt: row.wgt.unwrap_or(0) as i32,
                duration: row.duration.unwrap_or("".to_string()),
                addiction,
                quantity: 1,
            })
        }
    }
    result
}

pub fn resolve_consumable_type(ctype: i64) -> ConsumableType {
    match ctype {
        1 => ConsumableType::Chem,
        2 => ConsumableType::Food,
        3 => ConsumableType::Beverage,
        4 => ConsumableType::Other,
        5 => ConsumableType::Publication,
        _ => ConsumableType::Other,
    }
}

fn resolve_robot_modules(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
) -> Vec<RobotModule> {
    let mut result: Vec<RobotModule> = vec![];
    let selected_rmod_ids: Vec<i32> = background.robot_module_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (RobotModuleSelSlot::Fixed(opt), _) => vec![opt.bg_module_id],
            (RobotModuleSelSlot::Choice(opts), SlotSelection::Chosen(i)) => vec![opts[*i].bg_module_id],
            _ => vec![],
        })
        .collect();
    if selected_rmod_ids.is_empty() { return vec![] } 

    let id_json = serde_json::to_string(&selected_rmod_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                br.id        AS bg_rmod_id,
                r.id         AS id,
                r.name       AS name,
                r.eff        AS effs,
                r.wgt        AS wgt
                FROM background_robot_modules br
            JOIN robot_modules r ON r.id  = br.robot_module_id
            WHERE br.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        result.push( RobotModule {
            id: row.id.unwrap_or(0) as i32,
            name: row.name.unwrap_or("".to_string()),
            installed: false,
            effect: vec![row.effs.unwrap_or("".to_string())],
            wgt: row.wgt.unwrap_or(0) as i32,
        })
    }
    result
}

fn resolve_remaining_eq(
    db: &Db,
    background: &ResolvedBackground,
) -> (Vec<Gear>, Junk, Vec<String>) {
    let mut g_result: Vec<Gear> = vec![];

    let selected_gear_ids: Vec<i32> = background.gear.iter().map(|g| g.gear_id).collect();
    let id_json = serde_json::to_string(&selected_gear_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                bg.id        AS bg_gear_id,
                g.id         AS id,
                g.name       AS name,
                g.eff        AS effs,
                g.wgt        AS wgt
            FROM background_gear bg
            JOIN gear g ON g.id  = bg.gear_id
            WHERE bg.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        if g_result.iter().any(|g| g.id == row.bg_gear_id.unwrap_or(0) as i32) {
            let g_loc = g_result.iter().position(|g| g.id == row.bg_gear_id.unwrap_or(0) as i32);
            g_result[g_loc.unwrap()].quantity += 1;
        } else {
            g_result.push(Gear {
                id: row.id.unwrap_or(0) as i32,
                name: row.name.unwrap_or("".to_string()),
                effect: vec![row.effs.unwrap_or("".to_string())],
                wgt: row.wgt.unwrap_or(0) as i32,
                quantity: 1,
            })
        }
    }
    let junk = Junk {
        common: roll_cd(&format!("{}CD",background.junk)),
        uncommon: 0,
        rare: 0,
    };
    (g_result, junk, vec![background.misc.clone()])
}

pub fn roll_cd(roll_str: &str) -> i32 {
    let mut val = 0;
    let mut roll = 0;
    if let Some(plus_pos) = roll_str.find('+') {
        let before = &roll_str[..plus_pos];
        let after = &roll_str[plus_pos + 1..];
        if let Some(num) = before
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<i32>().ok())
            .next()
        {
            val = num;
        }
        if let Some(num) = after
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<i32>().ok())
            .next()
        {
            roll = num;
        }
    } else {
        if let Some(cd_pos) = roll_str.find('c') {
            let cd_str = &roll_str[..cd_pos];
            if let Some(num) = cd_str
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<i32>().ok())
                .next()
            {
                roll = num;
            }
        } else if let Some(num) = roll_str
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<i32>().ok())
            .next()
        {
            val = num
        }
    }
    let mut result = val;
    for _ in 0..roll { 
        let cd = rand::random_range(0..6);
        result += match cd {
            0..2 => 0,
            2..5 => 1,
            5 => 2,
            _ => 0, //stupid linter bitching at me thinking i missed a possible int
        }
    }
    result
}

fn selections_complete(sels: &[SlotSelection]) -> bool {
    sels.iter().all(|s| match s {
        SlotSelection::Fixed => true,
        SlotSelection::Chosen(i) => *i != usize::MAX,
        SlotSelection::ManyForOneChosen(i) => *i != usize::MAX,
        SlotSelection::SingleOrDoubleChosen(take_single, double_picks) => *take_single || double_picks.iter().all(|p| p.is_some()),
        SlotSelection::SingleOrPackChosen(_) => true,
    })
}

//handling all the options and slots
#[derive(Debug, Clone)]
pub struct WeaponOption {
    pub bg_weapon_id: i32,
    pub weapon_id: i32,
    pub name: String,
    pub mod_name: Option<String>,
    pub extra_mods: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum WeaponSelSlot {
    Fixed(WeaponOption),
    Choice(Vec<WeaponOption>),
    ManyForOne(Vec<WeaponOption>, WeaponOption),
}

#[derive(Debug, Clone)]
pub struct ApparelOption {
    pub bg_apparel_id: i32,
    pub apparel_id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ApparelSelSlot {
    Fixed(ApparelOption),
    Choice(Vec<ApparelOption>),
    SingleOrDouble(ApparelOption,Vec<Vec<ApparelOption>>),
    SingleOrPack(ApparelOption,Vec<ApparelOption>),
}

#[derive(Debug, Clone)]
pub struct ConsumableOption {
    pub bg_consumable_id: i32,
    pub consumable_id: i32,
    pub name: String,
    pub wgt: i32,
}

#[derive(Debug, Clone)]
pub enum ConsumableSelSlot {
    Fixed(ConsumableOption),
    Choice(Vec<ConsumableOption>),
    ManyForOne(Vec<ConsumableOption>, ConsumableOption),
}

#[derive(Debug, Clone)]
pub struct RobotModuleOption {
    pub bg_module_id: i32,
    pub module_id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum RobotModuleSelSlot {
    Fixed(RobotModuleOption),
    Choice(Vec<RobotModuleOption>),
}

#[derive(Debug, Clone)]
pub struct ResolvedBackground {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub weapon_slots:   Vec<WeaponSelSlot>,
    pub apparel_slots:  Vec<ApparelSelSlot>,
    pub consumable_slots: Vec<ConsumableSelSlot>,
    pub robot_module_slots: Vec<RobotModuleSelSlot>,
    pub ammo:  Vec<AmmoRow>,
    pub gear:  Vec<GearRow>,
    pub caps:    i32,
    pub misc:    String,
    pub trinket: i32,
    pub food:    i32,
    pub forage:  i32,
    pub bev:     i32,
    pub chem:    i32,
    pub ammo_count: i32,
    pub aid:     i32,
    pub odd:     i32,
    pub outcast: i32,
    pub junk:    i32,
}

//used to handle selection of options, covers all the possible selection types
#[derive(Debug, Clone)]
pub enum SlotSelection {
    Fixed,
    Chosen(usize),
    ManyForOneChosen(usize),
    SingleOrDoubleChosen(bool,Vec<Option<usize>>),
    SingleOrPackChosen(bool),
}

trait IsFixed { fn is_fixed(&self) -> bool; }
impl IsFixed for WeaponSelSlot {
    fn is_fixed(&self) -> bool { matches!(self, WeaponSelSlot::Fixed(_)) }
}
impl IsFixed for ConsumableSelSlot {
    fn is_fixed(&self) -> bool { matches!(self, ConsumableSelSlot::Fixed(_)) }
}
impl IsFixed for RobotModuleSelSlot {
    fn is_fixed(&self) -> bool { matches!(self, RobotModuleSelSlot::Fixed(_)) }
}

fn default_selections<T>(slots: &[T]) -> Vec<SlotSelection>
where T: IsFixed,
{
    slots.iter().map(|s| {
        if s.is_fixed() { SlotSelection::Fixed } else { SlotSelection::Chosen(usize::MAX) }
    }).collect()
}
fn default_apparel_selections(slots: &[ApparelSelSlot]) -> Vec<SlotSelection> {
    slots.iter().map(|s| match s {
        ApparelSelSlot::Fixed(_) => SlotSelection::Fixed,
        ApparelSelSlot::Choice(_) => SlotSelection::Chosen(usize::MAX),
        ApparelSelSlot::SingleOrDouble(_,double_choices) => SlotSelection::SingleOrDoubleChosen(true,vec![None; double_choices.len()],),
        ApparelSelSlot::SingleOrPack(..) => SlotSelection::SingleOrPackChosen(true),
    }).collect()
}

//we may want to just feed a full-blown weapon, rather than this option, for ease of implementation into the character. same with the apparel.
fn weapon_option(row: &WeaponRow) -> WeaponOption {
    WeaponOption {
        bg_weapon_id: row.id,
        weapon_id: row.weapon_id,
        name: row.weapon_name.clone(),
        mod_name: row.mod_name.clone(),
        extra_mods: vec![],
    }
}
fn apparel_option(row: &ApparelRow) -> ApparelOption {
    ApparelOption {
        bg_apparel_id: row.id,
        apparel_id: row.apparel_id,
        name: row.apparel_name.clone(),
    }
}

//the big block: translating the db content to choices
pub fn resolve_weapon_slots(rows: Vec<WeaponRow>) -> Vec<WeaponSelSlot> {
    //this block helps us understand which type of choice we need to present
    //maps each weapon to its professed alternate
    let mut fwd: HashMap<i32, i32> = HashMap::new();
    //maps each weapon to the ones that point to it
    let mut rev: HashMap<i32, Vec<i32>> = HashMap::new();
    //create a hashtable so we can quickly retrieve weaponrow data by its id, since we should only be looking at options for the current background (so ids don't match indices)
    let by_id: HashMap<i32, &WeaponRow> = rows.iter().map(|r| (r.id, r)).collect();

    //build the hashmaps
    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
    }

    //this is where we define all the options
    let mut slots: Vec<WeaponSelSlot> = vec![];
    //this tracks what we've already looked at (so we don't endlessly recurse, or just waste cycles reviewing stuff we've already assigned)
    let mut visited: HashSet<i32> = HashSet::new();

    //right now we're iterating twice, once for no-choice items and once for choices. i'm not sure if it's more efficient to only iterate once, or how much gains there are doing it that way

    //looks for weapons that don't have an alternate (so the character just gets it)
    for row in &rows {
        //doesn't have an alt, isn't pointed to by another weapon
        if row.alt_id.is_none() && !rev.contains_key(&row.id) {
            //hasn't been looked at yet
            if !visited.contains(&row.id) {
                //handle the one edge case where a weapon has two mods, so we need to consolidate the two rows into one weapon
                let same_weapon: Vec<&WeaponRow> = rows.iter()
                    .filter(
                        //finding all the weapons with the same id...
                        |r| r.weapon_id == row.weapon_id &&
                        //...that also have a mod...
                        r.mod_id.is_some() &&
                        //...and don't have a defined alt
                        r.alt_id.is_none()
                    )
                    .collect();
                if same_weapon.len() > 1 {
                    for r in &same_weapon {
                        visited.insert(r.id);
                    }
                    let mut opt = weapon_option(same_weapon[0]);
                    opt.extra_mods = same_weapon[1..].iter()
                        .filter_map(|r| r.mod_name.clone())
                        .collect();
                    slots.push(WeaponSelSlot::Fixed(opt));
                } else {
                    visited.insert(row.id);
                    slots.push(WeaponSelSlot::Fixed(weapon_option(row)));
                }
            }
        }
    }

    //checking for choices (pick 1 of 2+ options)
    let mut choice_visited: HashSet<i32> = HashSet::new();
    for row in &rows {
        //avoiding recursion/infinite loops
        if visited.contains(&row.id) || choice_visited.contains(&row.id) {
            continue;
        }
        //if there is no alt, there's no need to bother with this row
        //this might actually be redundant, since we should have already moved these into visited from the last for loop
        if row.alt_id.is_none() { continue; }
        //gathering the cycle
        let mut cycle: Vec<i32> = vec![];
        let mut current = row.id;
        loop {
            //we know that we've looked at everything at this point
            if cycle.contains(&current) || choice_visited.contains(&current) { break; }
            //add the current row to the cycle
            cycle.push(current);
            //track that we've looked at this row
            choice_visited.insert(current);
            //if the current row is in fwd, move on to the alt identified by fwd
            match fwd.get(&current) {
                Some(&next) => current = next,
                None => break,
            }
            //essentially, we "follow the loop" of alts, no matter how many choices there are in the loop
        }
        //there's a chance that one of our options should actually be two weapons
        //the second weapon will be inherently left out if we just follow the loops, since weapon one points to the next option in the line (not its partner), and the last option in the line points to weapon one
        let many_target = cycle.iter().find(|&&id| {
            //checking each weapon in the cycle to see if multiple weapons point to it (this would be both weapon one and weapon two)
            rev.get(&id).map(|v| v.len() > 1).unwrap_or(false)
        });
        if let Some(&target) = many_target {
            //linguistically we're calling this "giving up" two weapons for the single-weapon options
            //this might not be accounting for a pick 2 or 1 of 2+
            let give_up: Vec<WeaponOption> = rev[&target].iter()
                .filter_map(
                    |&id| by_id
                        .get(&id)
                        .map(|r| weapon_option(r))
                )
                .collect();
            let get = weapon_option(by_id[&target]);
            slots.push(WeaponSelSlot::ManyForOne(give_up, get))
        } else {
            let options: Vec<WeaponOption> = cycle.iter()
                .filter_map(
                    |id| by_id
                        .get(id)
                        .map(|r| weapon_option(r))
                )
                .collect();
            //this also might be redundant? not sure we should see this ever come up
            if options.len() == 1 {
                slots.push(WeaponSelSlot::Fixed(options.into_iter().next().unwrap()));
            } else {
                slots.push(WeaponSelSlot::Choice(options));
            }
        }
    }
    slots
}

fn weapon_label(opt: &WeaponOption) -> String {
    let mut s = opt.name.clone();
    if let Some(m) = &opt.mod_name { s.push_str(&format!(" w/ {}", m)); }
    if !opt.extra_mods.is_empty() {
        s.push_str(&format!(" + {}", opt.extra_mods.join(", ")));
    }
    s
}

fn render_weapon_option_label(ui: &Ui, opt: &WeaponOption) {
    ui.text(format!("  {}", weapon_label(opt)))
}

fn render_ammo_for(ui: &Ui, bg_weapon_id: i32, ammo: &[AmmoRow]) {
    for a in ammo.iter().filter(|a| a.bg_weapon_id == bg_weapon_id) {
        ui.text_disabled(format!("    Ammo: {} ({})", a.ammo_name, a.quantity));
    }
}

fn render_weapon_slot(ui: &Ui, index: usize, slot: &WeaponSelSlot, sel: &mut SlotSelection, ammo: &[AmmoRow]) -> bool {
    match slot {
        WeaponSelSlot::Fixed(opt) => {
            render_weapon_option_label(ui, opt);
            render_ammo_for(ui, opt.bg_weapon_id, ammo);
        }
        WeaponSelSlot::Choice(opts) => {
            let chosen_index = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_index < opts.len() {
                weapon_label(&opts[chosen_index])
            } else {
                format!("Weapon {} - choose...", index + 1)
            };
            ui.set_next_item_width(300.0);
            if let Some(_cb) = ui.begin_combo(format!("##wslot_{}", index), &preview) {
                for (oi, opt) in opts.iter().enumerate() {
                    let s = chosen_index == oi;
                    if ui.selectable_config(&weapon_label(opt)).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
            if chosen_index < opts.len() {
                render_ammo_for(ui, opts[chosen_index].bg_weapon_id, ammo);
            }
        }
        WeaponSelSlot::ManyForOne(give_up, get_one) => {
            let chosen = if let SlotSelection::ManyForOneChosen(i) = sel { *i } else { 0 };
            ui.text(format!("Choose: take all of {} OR just {}", weapon_label(get_one), give_up.iter().map(|w| weapon_label(w)).collect::<Vec<_>>().join(" + ")));
            //let mut take_one = chosen == 0;
            let take_one = chosen == 0;
            if ui.radio_button_bool(format!("Take all of {}##mfo_one_{}", weapon_label(get_one), index), take_one) {
                *sel = SlotSelection::ManyForOneChosen(0);
            }
            ui.same_line();
            if ui.radio_button_bool(
                format!("Take just {}##mfo_many_{}", give_up.iter().map(|w| weapon_label(w)).collect::<Vec<_>>().join("+"), index),
                !take_one
            ) {
                *sel = SlotSelection::ManyForOneChosen(1);
            }
        }
    }
    true
}

pub fn resolve_apparel_slots(rows: Vec<ApparelRow>) -> Vec<ApparelSelSlot> {
    //this block helps us understand which type of choice we need to present
    //maps each apparel to its professed alternate
    let mut fwd: HashMap<i32, i32> = HashMap::new();
    //maps each apparel to the ones that point to it
    let mut rev: HashMap<i32, Vec<i32>> = HashMap::new();
    //create a hashtable so we can quickly retrieve apparelrow data by its id, since we should only be looking at options for the current background (so ids don't match indices)
    let by_id: HashMap<i32, &ApparelRow> = rows.iter().map(|r| (r.id, r)).collect();
    let mut apparel_id_count: HashMap<i32, Vec<i32>> = HashMap::new();

    //build the hashmaps
    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
        apparel_id_count.entry(row.apparel_id).or_default().push(row.id);
    }

    //we handle apparel a little differently because of the nature of the choices, so we're capturing all the situations where the same apparel shows up multiple times in one background
    //this is kind of like how we had to deal with weapon mods, except it's not joining mods, it's identifying it as one option with multiple alts
    let repeated: HashSet<i32> = apparel_id_count.iter()
        .filter(|(_, ids)| ids.len() > 1)
        .flat_map(|(_, ids)| ids.iter().copied())
        .collect();
    // also making sure we don't count the double choices as their own individual choices
    let repeated_alts: HashSet<i32> = rows.iter()
        .filter(|r| repeated.contains(&r.id))
        .filter_map(|r| r.alt_id)
        .collect();

    //this is where we define all the options
    let mut slots: Vec<ApparelSelSlot> = vec![];
    //this tracks what we've already looked at (so we don't endlessly recurse, or just waste cycles reviewing stuff we've already assigned)
    let mut visited: HashSet<i32> = HashSet::new();

    //handle all the items that don't have alternates
    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) && !repeated.contains(&row.id) {
            if visited.insert(row.id) {
                slots.push(ApparelSelSlot::Fixed(apparel_option(row)));
            }
        }
    }

    //apparel choices can be a normal 1 of 2+, but they can also be a "one or pick 2" or "one or multiple", such as "leather torso or one leather arm and one leather leg"
    //this finds that "anchor" item; in the example, "leather torso" would be the anchor
    let single_anchor: Option<i32> = apparel_id_count.iter()
        .find(|(_, ids)| ids.len() > 1)
        .map(|(_,ids)| ids[0]);

    //get the normal cycles like with weapons
    let mut choice_visited: HashSet<i32> = HashSet::new();
    for row in &rows {
        //make sure we haven't looked at it already
        if visited.contains(&row.id) || choice_visited.contains(&row.id) {continue;}
        //make sure it's not one of the weird choices
        if repeated.contains(&row.id) {continue;}
        //make sure it's not one of the double choices
        if repeated_alts.contains(&row.id) {continue;}
        //make sure it isn't a fixed item (this probably never triggers?)
        if row.alt_id.is_none() {continue;}

        //building the alt loop
        let mut cycle: Vec<i32> = vec![];
        let mut current = row.id;
        loop {
            //this is how we know we have finished the loop
            if cycle.contains(&current) || choice_visited.contains(&current) {break;}
            cycle.push(current);
            choice_visited.insert(current);
            match fwd.get(&current) {
                //find the alt id of the current id in fwd and go there next
                Some(&next) => current = next,
                None => break,
            }
        }
        //build out our standard choice option
        let options: Vec<ApparelOption> = cycle.iter()
            .filter_map(
                |id| by_id
                    .get(id)
                    .map(|r| apparel_option(r))
            )
            .collect();
        //again, i'm not sure that this should actually trigger
        if options.len() == 1 {
            slots.push(ApparelSelSlot::Fixed(options.into_iter().next().unwrap()));
        } else {
            slots.push(ApparelSelSlot::Choice(options));
        }
    }

    let pack_items: Vec<i32> = repeated_alts.iter()
        .filter(|&&id| {
            by_id.get(&id)
                .and_then(|r| r.alt_id)
                .map(|alt| repeated.contains(&alt))
                .unwrap_or(false)
        })
        .copied()
        .collect();
    let is_single_or_pack = !pack_items.is_empty();

    //this is where we actually handle the weird choices
    if let Some(anchor_id) = single_anchor {
        let anchor_row = by_id[&anchor_id];
        let single_opt = apparel_option(anchor_row);
        //checking if it's either to the pick 1 of 2 twice or pick a pack
        if is_single_or_pack {
            let pack: Vec<ApparelOption> = pack_items.iter()
                .filter_map(|id| by_id.get(id).map(|r| apparel_option(r)))
                .collect();
            slots.push(ApparelSelSlot::SingleOrPack(single_opt, pack))
        } else {
            let sibling_ids = &apparel_id_count[&anchor_row.apparel_id];
            //grab all the alt targets
            let sibling_alts: Vec<i32> = sibling_ids.iter()
                .filter_map(|id| fwd.get(id).copied())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            //resolve the choice groups (same as the standard options)
            let double_choices: Vec<Vec<ApparelOption>> = sibling_alts.iter().map(|&start| {
                let mut cycle: Vec<i32> = vec![];
                let mut current = start;
                loop {
                    if cycle.contains(&current) { break }
                    cycle.push(current);
                    match fwd.get(&current) {
                        Some(&next) => current = next,
                        None => break,
                    }
                }
                //does some dedupe to make sure we don't accidentally include options from other groups
                cycle.iter()
                    .filter_map(
                        |id| by_id
                            .get(id)
                            .map(|r| apparel_option(r))
                    )
                    .collect()
            }).collect();

            //does some further dedupe to make sure that we don't provide all the options in a double vs the two options for each
            let mut seen_sets: Vec<HashSet<i32>> = vec![];
            let mut deduped_choices: Vec<Vec<ApparelOption>> = vec![];
            for group in double_choices {
                let id_set: HashSet<i32> = group.iter()
                    .map(|o| o.bg_apparel_id)
                    .collect();
                if !seen_sets.iter().any(|s| s == &id_set) {
                    seen_sets.push(id_set);
                    deduped_choices.push(group);
                }
            }
            slots.push(ApparelSelSlot::SingleOrDouble(single_opt, deduped_choices));
        }
    }
    slots
}

fn render_apparel_slot(ui: &Ui, index: usize, slot: &ApparelSelSlot, sel: &mut SlotSelection) -> bool {
    match slot {
        ApparelSelSlot::Fixed(opt) => {
            ui.text(format!("  {}", opt.name));
        }
        ApparelSelSlot::Choice(opts) => {
            let chosen_index = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_index < opts.len() {
                opts[chosen_index].name.clone()
            } else {
                format!("Apparel {} - choose...", index + 1)
            };
            ui.set_next_item_width(300.0);
            if let Some(_cb) = ui.begin_combo(format!("##aslot_{}", index), &preview) {
                for (oi, opt) in opts.iter().enumerate() {
                    let s = chosen_index == oi;
                    if ui.selectable_config(&opt.name).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
        }
        ApparelSelSlot::SingleOrDouble(single, double_choices) => {
            let (take_single, double_picks) = if let SlotSelection::SingleOrDoubleChosen(take_single, double_picks) = sel {
                (take_single, double_picks)
            } else { return true; };

            if ui.radio_button_bool(format!("Take {}##sd_single_{}", single.name, index), *take_single) {
                *take_single = true;
                double_picks[0].take();
                double_picks[1].take();
            }
            ui.same_line();
            if ui.radio_button_bool(format!("Take two pieces##sd_double_{}", index), !*take_single) {
                *take_single = false;
            }
            if !*take_single {
                for (di, choices) in double_choices.iter().enumerate() {
                    let picked = double_picks[di];
                    let preview = picked
                        .map(|i| choices[i].name.clone())
                        .unwrap_or_else(|| format!("Slot {} - choose...", di + 1));
                    ui.set_next_item_width(280.0);
                    if let Some(_cb) = ui.begin_combo(format!("##adbl_{}_{}", index, di), &preview) {
                        for (oi, opt) in choices.iter().enumerate() {
                            let s = picked == Some(oi);
                            if ui.selectable_config(&opt.name).selected(s).build() {
                                double_picks[di] = Some(oi);
                            }
                        }
                    }
                }
            }
        }
        ApparelSelSlot::SingleOrPack(single, pack) => {
            let take_single = if let SlotSelection::SingleOrPackChosen(b) = sel { b } else { return true; };
            if ui.radio_button_bool(format!("Take just {}##sp_single_{}", single.name, index), *take_single) {
                *take_single = true;
            }
            ui.same_line();
            let pack_label = pack.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(" + ");
            if ui.radio_button_bool(format!("Take all of: {}##sp_pack_{}", pack_label, index), !*take_single) {
                *take_single = false;
            }
        }
    }
    true
}

pub fn resolve_consumable_slots(rows: Vec<ConsumableRow>) -> Vec<ConsumableSelSlot> {
    //this is very similar to the weapon stuff, but without the edge cases
    let mut fwd: HashMap<i32, i32> = HashMap::new();
    let mut rev: HashMap<i32, Vec<i32>> = HashMap::new();
    let by_id: HashMap<i32, &ConsumableRow> = rows.iter().map(|r| (r.id, r)).collect();

    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
    }

    let mut slots: Vec<ConsumableSelSlot> = vec![];
    let mut visited: HashSet<i32> = HashSet::new();

    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) {
            if visited.insert(row.id) {
                slots.push(ConsumableSelSlot::Fixed(ConsumableOption {
                    bg_consumable_id: row.id,
                    consumable_id: row.consumable_id,
                    name: row.consumable_name.clone(),
                    wgt: row.wgt,
                }));
            }
        }
    }

    let mut choice_visited: HashSet<i32> = HashSet::new();
    for row in &rows {
        if visited.contains(&row.id) || choice_visited.contains(&row.id) { continue; }
        if row.alt_id.is_none() { continue; }

        let mut cycle: Vec<i32> = vec![];
        let mut current = row.id;
        loop {
            if cycle.contains(&current) || choice_visited.contains(&current) { break; }
            cycle.push(current);
            choice_visited.insert(current);
            match fwd.get(&current) {
                Some(&next) => current = next,
                None => break,
            }
        }

        //our only weird edge case is the nuka options from the nukatron
        let many_target = cycle.iter().find(|&&id| {
            rev.get(&id).map(|v| v.len() > 1).unwrap_or(false)
        });

        if let Some(&target) = many_target {
            let give_up: Vec<ConsumableOption> = rev[&target].iter()
                .filter_map(|&id| by_id.get(&id))
                .map(|r| ConsumableOption {
                    bg_consumable_id: r.id,
                    consumable_id: r.consumable_id,
                    name: r.consumable_name.clone(),
                    wgt: row.wgt,
                })
                .collect();
            let get = ConsumableOption {
                bg_consumable_id: target,
                consumable_id: by_id[&target].consumable_id,
                name: by_id[&target].consumable_name.clone(),
                wgt: row.wgt,
            };
            slots.push(ConsumableSelSlot::ManyForOne(give_up, get));
        } else {
            let options: Vec<ConsumableOption> = cycle.iter()
                .filter_map(|id| by_id.get(id))
                .map(|r| ConsumableOption {
                    bg_consumable_id: r.id,
                    consumable_id: r.consumable_id,
                    name: r.consumable_name.clone(),
                    wgt: row.wgt,
                })
                .collect();
            if options.len() == 1 {
                slots.push(ConsumableSelSlot::Fixed(options.into_iter().next().unwrap()));
            } else {
                slots.push(ConsumableSelSlot::Choice(options));
            }
        }
    }
    slots
}

fn render_consumable_slot(ui: &Ui, index: usize, slot: &ConsumableSelSlot, sel: &mut SlotSelection) -> bool {
    match slot {
        ConsumableSelSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        ConsumableSelSlot::Choice(opts) => {
            let chosen_index = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_index < opts.len() {
                opts[chosen_index].name.clone()
            } else {
                format!("Consumable {} - choose...", index + 1)
            };
            ui.set_next_item_width(280.0);
            if let Some(_cb) = ui.begin_combo(format!("##cslot_{}", index), &preview) {
                for (oi, opt) in opts.iter().enumerate() {
                    let s = chosen_index == oi;
                    if ui.selectable_config(&opt.name).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
        }
        ConsumableSelSlot::ManyForOne(give_up, get_one) => {
            let chosen = if let SlotSelection::ManyForOneChosen(i) = sel { *i } else { 2 };
            ui.text(format!("Choose: take all of {} OR just {}",
                get_one.name,
                give_up.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(" + "),
            ));
            if ui.radio_button_bool(format!("Take just {}##cmfo_one_{}", get_one.name, index), chosen == 0) {
                *sel = SlotSelection::ManyForOneChosen(0);
            }
            ui.same_line();
            if ui.radio_button_bool(format!("Take all of ##cmfo_many_{}", index), chosen == 1) {
                *sel = SlotSelection::ManyForOneChosen(1);
            }
        }
    }
    true
}

//robot module is even simpler
pub fn resolve_robot_module_slots(rows: Vec<RobotModuleRow>) -> Vec<RobotModuleSelSlot> {
    let mut fwd: HashMap<i32, i32> = HashMap::new();
    let mut rev: HashMap<i32, Vec<i32>> = HashMap::new();
    let by_id: HashMap<i32, &RobotModuleRow> = rows.iter().map(|r| (r.id, r)).collect();

    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
    }

    let mut slots: Vec<RobotModuleSelSlot> = vec![];
    let mut visited: HashSet<i32> = HashSet::new();

    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) {
            if visited.insert(row.id) {
                slots.push(RobotModuleSelSlot::Fixed(RobotModuleOption {
                    bg_module_id: row.id, module_id: row.module_id,
                    name: row.module_name.clone(),
                }));
            }
        }
    }

    let mut choice_visited: HashSet<i32> = HashSet::new();
    for row in &rows {
        if visited.contains(&row.id) || choice_visited.contains(&row.id) { continue; }
        if row.alt_id.is_none() { continue; }
        let mut cycle: Vec<i32> = vec![];
        let mut current = row.id;
        loop {
            if cycle.contains(&current) || choice_visited.contains(&current) { break; }
            cycle.push(current);
            choice_visited.insert(current);
            match fwd.get(&current) {
                Some(&next) => current = next,
                None => break,
            }
        }
        let options: Vec<RobotModuleOption> = cycle.iter()
            .filter_map(|id| by_id.get(id))
            .map(|r| RobotModuleOption {
                bg_module_id: r.id, module_id: r.module_id,
                name: r.module_name.clone(),
            })
            .collect();
        if options.len() == 1 {
            slots.push(RobotModuleSelSlot::Fixed(options.into_iter().next().unwrap()));
        } else {
            slots.push(RobotModuleSelSlot::Choice(options));
        }
    }
    slots
}

fn render_robot_module_slot(ui: &Ui, index: usize, slot: &RobotModuleSelSlot, sel: &mut SlotSelection) -> bool {
    match slot {
        RobotModuleSelSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        RobotModuleSelSlot::Choice(opts) => {
            let chosen_index = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_index < opts.len() {
                opts[chosen_index].name.clone()
            } else {
                format!("Module {} - choose...", index + 1)
            };
            ui.set_next_item_width(280.0);
            if let Some(_cb) = ui.begin_combo(format!("##rmslot_{}", index), &preview) {
                for (oi, opt) in opts.iter().enumerate() {
                    let s = chosen_index == oi;
                    if ui.selectable_config(&opt.name).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
        }
    }
    true
}

pub fn render_background_select(
    ui: &Ui,
    window: &Window,
    state: &mut BackgroundState,
    equipment: &mut EquipmentState,
    db: &Db,
    character: &mut Character,
    _review: &mut ReviewState,
    screen: &mut AppScreen,
    origin: &mut OriginState,
    special: &mut SpecialState,
    skill: &mut SkillState,
    perk: &mut PerkState,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##background_select", "Background Select", screen, origin, special, skill, perk, state, equipment, character)
        else { return 0.0 };

    ui.text("BACKGROUND");
    ui.separator();
    ui.spacing();

    ui.text("Background:");
    ui.same_line();
    ui.set_next_item_width(280.0);
    //grab all the available backgrounds for the selected origin
    let bg_names: Vec<(usize, String)> = state.origin_backgrounds(character.clone())
        .into_iter()
        .map(|(i, bg)| (i, bg.name.clone()))
        .collect();
    let preview = state.selected_index
        .and_then(|i| state.all_backgrounds.get(i))
        .map(|bg| bg.name.as_str())
        .unwrap_or("Select background...");
    if let Some(_cb) = ui.begin_combo("##bg_select", preview) {
        for (i, name) in &bg_names {
            let sel = state.selected_index == Some(*i);
            if ui.selectable_config(name.as_str()).selected(sel).build() {
                if state.selected_index != Some(*i) {
                    state.load_background(db, *i);
                    character.background = Some(Background {
                        id: (*i + 1) as i32,
                        name: state.current_background.clone().unwrap().name,
                        desc: state.current_background.clone().unwrap().desc,
                    });
                }
            }
        }
    }
    ui.spacing();
    ui.separator();
    ui.spacing();
    //if there isn't a selected background, inform the player and stop rendering
    let Some(bg) = &state.current_background else {
        ui.text_disabled("Select a background to view starting equipment options...");
        return h;
    };
    //create a clone of the background we can reference
    //avoids borrowing issues
    let bg = bg.clone();
    //creates a scrolling child window for the selection (not sure if we ever need this much space but who knows)
    let list_h = h - 140.0;
    let Some(_child) = ui.child_window("##equip_scroll")
        .size([w - 16.0, list_h])
        .begin()
    else { return h };

    ui.separator();
    //weapons
    if !bg.weapon_slots.is_empty() {
        ui.text("WEAPONS");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.weapon_slots.iter().enumerate() {
            state.equipment_changed = render_weapon_slot(ui, i, slot, &mut state.weapon_selections[i], &bg.ammo);
            ui.spacing();
        }
        ui.spacing();
    }
    //apparel
    if !bg.apparel_slots.is_empty() {
        ui.text("APPAREL");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.apparel_slots.iter().enumerate() {
            state.equipment_changed = render_apparel_slot(ui, i, slot, &mut state.apparel_selections[i]);
            ui.spacing();
        }
        ui.spacing();
    }
    //consumables
    if !bg.consumable_slots.is_empty() {
        ui.text("CONSUMABLES");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.consumable_slots.iter().enumerate() {
            state.equipment_changed = render_consumable_slot(ui, i, slot, &mut state.consumable_selections[i]);
            ui.spacing();
        }
        ui.spacing();
    }
    //robot modules
    if !bg.robot_module_slots.is_empty() {
        ui.text("ROBOT MODULES");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.robot_module_slots.iter().enumerate() {
            state.equipment_changed = render_robot_module_slot(ui, i, slot, &mut state.robot_module_selections[i]);
            ui.spacing();
        }
        ui.spacing();
    }
    //gear
    if !bg.gear.is_empty() {
        ui.text("GEAR");
        ui.separator();
        ui.spacing();
        for g in &bg.gear {
            ui.text(format!("  {}", g.gear_name));
        }
        ui.spacing();
    }
    //misc
    ui.text("MISC");
    ui.separator();
    ui.spacing();
    ui.text(format!("  Caps: {}", bg.caps));
    if !bg.misc.is_empty()   { ui.text(format!("  Misc: {}", bg.misc)); }
    let mut table_vec: Vec<String> = vec![];
    if bg.trinket > 0  { table_vec.push(format!("  Trinket x{}", bg.trinket)); }
    if bg.food > 0     { table_vec.push(format!("  Food x{}", bg.food)); }
    if bg.forage > 0   { table_vec.push(format!("  Forage x{}", bg.forage)); }
    if bg.bev > 0      { table_vec.push(format!("  Beverages x{}", bg.bev)); }
    if bg.chem > 0     { table_vec.push(format!("  Chems x{}", bg.chem)); }
    if bg.ammo_count > 0  { table_vec.push(format!("  Ammo x{}", bg.ammo_count)); }
    if bg.aid > 0      { table_vec.push(format!("  Aid x{}", bg.aid)); }
    if bg.odd > 0      { table_vec.push(format!("  Oddities x{}", bg.odd)); }
    if bg.outcast > 0  { table_vec.push(format!("  Outcast Equipment x{}", bg.outcast)); }
    if bg.junk > 0     { table_vec.push(format!("  Junk x{}", bg.junk)); }
    if table_vec.len() > 0 {
        ui.text("Table Rolls:");
        for i in 0..table_vec.len() {
            ui.text(table_vec[i].clone());
        }
    }
    ui.spacing();
    ui.separator();
    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.text("debug output");
    ui.spacing();
    ui.separator();
    ui.spacing();

    /*
    ui.text_disabled("background:");
    ui.same_line();
    if state.current_background.is_none() {
        ui.text("none");
    } else {
        let bg = state.current_background.clone().unwrap();
        ui.text_wrapped(format!("  id: {}   name: {}   caps: {}   misc: {}", bg.id, bg.name, bg.caps, bg.misc));
        ui.text_wrapped(format!("              trinket: {}  food: {}  forage: {}  bev: {}  chem: {}  ammo: {}  aid: {}  odd: {}  outcast: {}  junk: {}", bg.trinket, bg.food, bg.forage, bg.bev, bg.chem, bg.ammo_count, bg.aid, bg.odd, bg.outcast, bg.junk));
        ui.text_disabled("  weapon_slots:      ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.weapon_slots));
        ui.text_disabled("  ammo:              ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.ammo));
        ui.text_disabled("  apparel_slots:     ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.apparel_slots));
        ui.text_disabled("  consumable_slots:  ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.consumable_slots));
        ui.text_disabled("  robotmod_slots:    ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.robot_module_slots));
        ui.text_disabled("  gear_slots:        ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.gear));
    }
    ui.text_disabled("weapons:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.weapon_selections));
    ui.text_disabled("apparel:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.apparel_selections));
    ui.text_disabled("consumables:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.consumable_selections));
    ui.text_disabled("robot modules:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.robot_module_selections));
    */

    ui.separator();
    ui.separator();
    ui.text_disabled("weapons   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.weapons));
    ui.text_disabled("ammo   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.ammo));
    ui.text_disabled("apparel   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.apparel));
    ui.text_disabled("consumables   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.consumables));
    ui.text_disabled("robot modules   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.robot_modules));
    ui.text_disabled("gear   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.gear));
    ui.text_disabled("junk   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.junk));
    ui.text_disabled("misc   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.misc));


    //ends the scroll window
    drop(_child);
    h
}