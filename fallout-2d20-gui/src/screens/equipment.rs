use crate::{db::Db, render_placeholder};
use std::collections::{HashMap, HashSet};
use imgui::Ui;
use sdl2::video::Window;
use crate::{AppScreen, BAR_HEIGHT};

// ── Raw DB rows (from sqlx queries) ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BackgroundRow {
    pub id: i64,
    pub origin_id: i64,
    pub name: String,
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
    pub id: i64,
    pub bg_id: i64,
    pub weapon_id: i64,
    pub weapon_name: String,
    pub mod_id: Option<i64>,
    pub mod_name: Option<String>,
    pub alt_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ApparelRow {
    pub id: i64,
    pub bg_id: i64,
    pub apparel_id: i64,
    pub apparel_name: String,
    pub alt_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ConsumableRow {
    pub id: i64,
    pub bg_id: i64,
    pub consumable_id: i64,
    pub consumable_name: String,
    pub alt_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct RobotModuleRow {
    pub id: i64,
    pub bg_id: i64,
    pub module_id: i64,
    pub module_name: String,
    pub alt_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct AmmoRow {
    pub ammo_id: i64,
    pub ammo_name: String,
    pub quantity: String,
    pub bg_weapon_id: i64,  // links to background_weapons.id
}

#[derive(Debug, Clone)]
pub struct GearRow {
    pub gear_id: i64,
    pub gear_name: String,
}

// ── Resolved choice model ─────────────────────────────────────────────────────
//
// After loading, each category is a Vec<ItemSlot>.
// A slot is either Fixed (no choice) or Choice (pick one option).
// An option is either a single item or a bundle of items (many-for-one swaps).

#[derive(Debug, Clone)]
pub struct WeaponOption {
    pub bg_weapon_id: i64,
    pub weapon_id: i64,
    pub name: String,
    pub mod_name: Option<String>,
    /// Multiple mods consolidated onto one weapon
    pub extra_mods: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum WeaponSlot {
    Fixed(WeaponOption),
    Choice(Vec<WeaponOption>),           // pick one
    ManyForOne(Vec<WeaponOption>, WeaponOption), // give up Vec, get Option
}

#[derive(Debug, Clone)]
pub struct ApparelOption {
    pub bg_apparel_id: i64,
    pub apparel_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ApparelSlot {
    Fixed(ApparelOption),
    Choice(Vec<ApparelOption>),          // pick one
    /// One item OR two items (left + right choice each)
    SingleOrDouble {
        single: ApparelOption,
        double_choices: Vec<Vec<ApparelOption>>, // one inner Vec per double slot
    },
    /// One item OR a pack of fixed items
    SingleOrPack {
        single: ApparelOption,
        pack: Vec<ApparelOption>,
    },
}

#[derive(Debug, Clone)]
pub struct ConsumableOption {
    pub bg_consumable_id: i64,
    pub consumable_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ConsumableSlot {
    Fixed(ConsumableOption),
    Choice(Vec<ConsumableOption>),
    ManyForOne(Vec<ConsumableOption>, ConsumableOption),
}

#[derive(Debug, Clone)]
pub struct RobotModuleOption {
    pub bg_module_id: i64,
    pub module_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum RobotModuleSlot {
    Fixed(RobotModuleOption),
    Choice(Vec<RobotModuleOption>),
}

#[derive(Debug, Clone)]
pub struct ResolvedBackground {
    pub id: i64,
    pub name: String,
    pub weapon_slots:   Vec<WeaponSlot>,
    pub apparel_slots:  Vec<ApparelSlot>,
    pub consumable_slots: Vec<ConsumableSlot>,
    pub robot_module_slots: Vec<RobotModuleSlot>,
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

// ── Weapon grouping ───────────────────────────────────────────────────────────

pub fn resolve_weapon_slots(rows: Vec<WeaponRow>) -> Vec<WeaponSlot> {
    // Build forward and reverse alt maps
    let mut fwd: HashMap<i64, i64> = HashMap::new();  // id → alt_id
    let mut rev: HashMap<i64, Vec<i64>> = HashMap::new(); // alt_id → [ids that point to it]
    let by_id: HashMap<i64, &WeaponRow> = rows.iter().map(|r| (r.id, r)).collect();

    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
    }

    // Items with no alt_id and not pointed to by anything = fixed
    // Items with alt_id = part of a choice group
    // Items pointed to by multiple = the "canonical" option in a many-for-one

    let mut slots: Vec<WeaponSlot> = vec![];
    let mut visited: HashSet<i64> = HashSet::new();

    // First pass: fixed items (no alt, not a target)
    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) {
            if !visited.contains(&row.id) {
                visited.insert(row.id);
                // Consolidate multi-mod weapons (same weapon_id, different mod_id)
                let same_weapon: Vec<&WeaponRow> = rows.iter()
                    .filter(|r| r.weapon_id == row.weapon_id && r.mod_id.is_some() && r.alt_id.is_none())
                    .collect();
                if same_weapon.len() > 1 {
                    // Already handled in multi-mod pass below
                    continue;
                }
                slots.push(WeaponSlot::Fixed(weapon_option(row)));
            }
        }
    }

    // Multi-mod consolidation: same weapon_id, multiple rows each with a different mod, no alt
    let mut multi_mod_seen: HashSet<i64> = HashSet::new();
    for row in &rows {
        if row.mod_id.is_some() && row.alt_id.is_none() && !rev.contains_key(&row.id) {
            if multi_mod_seen.contains(&row.weapon_id) { continue; }
            let group: Vec<&WeaponRow> = rows.iter()
                .filter(|r| r.weapon_id == row.weapon_id && r.mod_id.is_some() && r.alt_id.is_none() && !rev.contains_key(&r.id))
                .collect();
            if group.len() > 1 {
                multi_mod_seen.insert(row.weapon_id);
                for r in &group { visited.insert(r.id); }
                let mut opt = weapon_option(group[0]);
                opt.extra_mods = group[1..].iter()
                    .filter_map(|r| r.mod_name.clone())
                    .collect();
                slots.push(WeaponSlot::Fixed(opt));
            }
        }
    }

    // Choice groups: follow alt chains to find cycles
    let mut choice_visited: HashSet<i64> = HashSet::new();
    for row in &rows {
        if visited.contains(&row.id) || choice_visited.contains(&row.id) { continue; }
        if row.alt_id.is_none() { continue; }

        // Collect the full cycle
        let mut cycle: Vec<i64> = vec![];
        let mut cur = row.id;
        loop {
            if cycle.contains(&cur) || choice_visited.contains(&cur) { break; }
            cycle.push(cur);
            choice_visited.insert(cur);
            match fwd.get(&cur) {
                Some(&next) => cur = next,
                None => break,
            }
        }

        // Check if this is many-for-one: one node in cycle is pointed to by multiple
        let many_target = cycle.iter().find(|&&id| {
            rev.get(&id).map(|v| v.len() > 1).unwrap_or(false)
        });

        if let Some(&target) = many_target {
            // Many-for-one: the pointers to target are the "give up" set
            let give_up: Vec<WeaponOption> = rev[&target].iter()
                .filter_map(|&id| by_id.get(&id).map(|r| weapon_option(r)))
                .collect();
            let get = weapon_option(by_id[&target]);
            slots.push(WeaponSlot::ManyForOne(give_up, get));
        } else {
            let options: Vec<WeaponOption> = cycle.iter()
                .filter_map(|id| by_id.get(id).map(|r| weapon_option(r)))
                .collect();
            if options.len() == 1 {
                slots.push(WeaponSlot::Fixed(options.into_iter().next().unwrap()));
            } else {
                slots.push(WeaponSlot::Choice(options));
            }
        }
    }

    slots
}

fn weapon_option(row: &WeaponRow) -> WeaponOption {
    WeaponOption {
        bg_weapon_id: row.id,
        weapon_id: row.weapon_id,
        name: row.weapon_name.clone(),
        mod_name: row.mod_name.clone(),
        extra_mods: vec![],
    }
}

// ── Apparel grouping ──────────────────────────────────────────────────────────

pub fn resolve_apparel_slots(rows: Vec<ApparelRow>) -> Vec<ApparelSlot> {
    let mut fwd: HashMap<i64, i64> = HashMap::new();
    let mut rev: HashMap<i64, Vec<i64>> = HashMap::new();
    let by_id: HashMap<i64, &ApparelRow> = rows.iter().map(|r| (r.id, r)).collect();
    // Count how many bg rows share the same apparel_id
    let mut apparel_id_count: HashMap<i64, Vec<i64>> = HashMap::new();

    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
        apparel_id_count.entry(row.apparel_id).or_default().push(row.id);
    }

    // Repeated apparel_id = the "single" in a single/double or single/pack pattern
    let repeated: HashSet<i64> = apparel_id_count.iter()
        .filter(|(_, ids)| ids.len() > 1)
        .flat_map(|(_, ids)| ids.iter().copied())
        .collect();

    let mut slots: Vec<ApparelSlot> = vec![];
    let mut visited: HashSet<i64> = HashSet::new();

    // Fixed items first
    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) && !repeated.contains(&row.id) {
            if visited.insert(row.id) {
                slots.push(ApparelSlot::Fixed(apparel_option(row)));
            }
        }
    }

    // Find the single/double or single/pack anchor
    let single_anchor: Option<i64> = apparel_id_count.iter()
        .find(|(_, ids)| ids.len() > 1)
        .map(|(_, ids)| ids[0]);

    // Regular choice groups (excluding repeated)
    let mut choice_visited: HashSet<i64> = HashSet::new();
    for row in &rows {
        if visited.contains(&row.id) || choice_visited.contains(&row.id) { continue; }
        if repeated.contains(&row.id) { continue; }
        if row.alt_id.is_none() { continue; }

        let mut cycle: Vec<i64> = vec![];
        let mut cur = row.id;
        loop {
            if cycle.contains(&cur) || choice_visited.contains(&cur) { break; }
            cycle.push(cur);
            choice_visited.insert(cur);
            match fwd.get(&cur) {
                Some(&next) => cur = next,
                None => break,
            }
        }

        let options: Vec<ApparelOption> = cycle.iter()
            .filter_map(|id| by_id.get(id).map(|r| apparel_option(r)))
            .collect();
        if options.len() == 1 {
            slots.push(ApparelSlot::Fixed(options.into_iter().next().unwrap()));
        } else {
            slots.push(ApparelSlot::Choice(options));
        }
    }

    // Resolve single/double or single/pack
    if let Some(anchor_id) = single_anchor {
        let anchor_row = by_id[&anchor_id];
        let single_opt = apparel_option(anchor_row);

        // Check if any repeated item links to a choice group = single/double
        // Check if any repeated item links to a pack (chain of fixed) = single/pack
        let sibling_ids = &apparel_id_count[&anchor_row.apparel_id];

        // The non-anchor siblings' alt_ids point to the double choices or pack items
        let sibling_alts: Vec<i64> = sibling_ids.iter()
            .filter(|&&id| id != anchor_id)
            .filter_map(|id| fwd.get(id).copied())
            .collect();
        eprintln!("anchor_id={anchor_id} sibling_ids={sibling_ids:?} sibling_alts={sibling_alts:?}");

        // Determine if siblings' alts form choice groups (double) or fixed chains (pack)
        let is_pack = sibling_alts.iter().all(|&alt_id| {
            !fwd.contains_key(&alt_id) // alt has no further alt = fixed pack item
        });

        if is_pack {
            let pack: Vec<ApparelOption> = sibling_alts.iter()
                .filter_map(|id| by_id.get(id).map(|r| apparel_option(r)))
                .collect();
            slots.push(ApparelSlot::SingleOrPack { single: single_opt, pack });
        } else {
            // Double: each sibling alt is a choice group
            let double_choices: Vec<Vec<ApparelOption>> = sibling_alts.iter().map(|&start| {
                let mut cyc: Vec<i64> = vec![];
                let mut cur = start;
                loop {
                    if cyc.contains(&cur) { break; }
                    cyc.push(cur);
                    match fwd.get(&cur) {
                        Some(&next) => cur = next,
                        None => break,
                    }
                }
                cyc.iter().filter_map(|id| by_id.get(id).map(|r| apparel_option(r))).collect()
            }).collect();
            slots.push(ApparelSlot::SingleOrDouble { single: single_opt, double_choices });
        }
    }

    slots
}

fn apparel_option(row: &ApparelRow) -> ApparelOption {
    ApparelOption {
        bg_apparel_id: row.id,
        apparel_id: row.apparel_id,
        name: row.apparel_name.clone(),
    }
}

// ── Consumable grouping (same pattern as weapons, no many-for-one) ────────────

pub fn resolve_consumable_slots(rows: Vec<ConsumableRow>) -> Vec<ConsumableSlot> {
    let mut fwd: HashMap<i64, i64> = HashMap::new();
    let mut rev: HashMap<i64, Vec<i64>> = HashMap::new();
    let by_id: HashMap<i64, &ConsumableRow> = rows.iter().map(|r| (r.id, r)).collect();

    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
    }

    let mut slots: Vec<ConsumableSlot> = vec![];
    let mut visited: HashSet<i64> = HashSet::new();

    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) {
            if visited.insert(row.id) {
                slots.push(ConsumableSlot::Fixed(ConsumableOption {
                    bg_consumable_id: row.id,
                    consumable_id: row.consumable_id,
                    name: row.consumable_name.clone(),
                }));
            }
        }
    }

    let mut choice_visited: HashSet<i64> = HashSet::new();
    for row in &rows {
        if visited.contains(&row.id) || choice_visited.contains(&row.id) { continue; }
        if row.alt_id.is_none() { continue; }

        let mut cycle: Vec<i64> = vec![];
        let mut cur = row.id;
        loop {
            if cycle.contains(&cur) || choice_visited.contains(&cur) { break; }
            cycle.push(cur);
            choice_visited.insert(cur);
            match fwd.get(&cur) {
                Some(&next) => cur = next,
                None => break,
            }
        }

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
                })
                .collect();
            let get = ConsumableOption {
                bg_consumable_id: target,
                consumable_id: by_id[&target].consumable_id,
                name: by_id[&target].consumable_name.clone(),
            };
            slots.push(ConsumableSlot::ManyForOne(give_up, get));
        } else {
            let options: Vec<ConsumableOption> = cycle.iter()
                .filter_map(|id| by_id.get(id))
                .map(|r| ConsumableOption {
                    bg_consumable_id: r.id,
                    consumable_id: r.consumable_id,
                    name: r.consumable_name.clone(),
                })
                .collect();
            if options.len() == 1 {
                slots.push(ConsumableSlot::Fixed(options.into_iter().next().unwrap()));
            } else {
                slots.push(ConsumableSlot::Choice(options));
            }
        }
    }

    slots
}

// ── Robot module grouping (simple choices only) ───────────────────────────────

pub fn resolve_robot_module_slots(rows: Vec<RobotModuleRow>) -> Vec<RobotModuleSlot> {
    let mut fwd: HashMap<i64, i64> = HashMap::new();
    let mut rev: HashMap<i64, Vec<i64>> = HashMap::new();
    let by_id: HashMap<i64, &RobotModuleRow> = rows.iter().map(|r| (r.id, r)).collect();

    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
    }

    let mut slots: Vec<RobotModuleSlot> = vec![];
    let mut visited: HashSet<i64> = HashSet::new();

    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) {
            if visited.insert(row.id) {
                slots.push(RobotModuleSlot::Fixed(RobotModuleOption {
                    bg_module_id: row.id, module_id: row.module_id,
                    name: row.module_name.clone(),
                }));
            }
        }
    }

    let mut choice_visited: HashSet<i64> = HashSet::new();
    for row in &rows {
        if visited.contains(&row.id) || choice_visited.contains(&row.id) { continue; }
        if row.alt_id.is_none() { continue; }
        let mut cycle: Vec<i64> = vec![];
        let mut cur = row.id;
        loop {
            if cycle.contains(&cur) || choice_visited.contains(&cur) { break; }
            cycle.push(cur);
            choice_visited.insert(cur);
            match fwd.get(&cur) {
                Some(&next) => cur = next,
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
            slots.push(RobotModuleSlot::Fixed(options.into_iter().next().unwrap()));
        } else {
            slots.push(RobotModuleSlot::Choice(options));
        }
    }
    slots
}

pub fn load_backgrounds(db: &Db) -> Vec<BackgroundRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"SELECT id, origin_id, name, caps, misc, trinket, food, forage,
               bev, chem, ammo, aid, odd, outcast, junk
               FROM backgrounds ORDER BY id"#
        )
        .fetch_all(&db.pool).await
    });
    match result {
        Ok(rows) => rows.into_iter().map(|r| BackgroundRow {
            id: r.id, name: r.name.unwrap_or_default(),
            origin_id: r.origin_id.unwrap_or_default(),
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

pub fn load_background_equipment(db: &Db, background_id: i64) -> ResolvedBackground {
    // Load background row
    let bg = load_backgrounds(db).into_iter()
        .find(|b| b.id == background_id)
        .unwrap_or_else(|| BackgroundRow {
            id: background_id, name: String::new(), origin_id: 0,
            caps: 0, misc: String::new(),
            trinket: 0, food: 0, forage: 0, bev: 0, chem: 0,
            ammo: 0, aid: 0, odd: 0, outcast: 0, junk: 0,
        });

    // Weapons
    let weapon_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bw.id, bw.background_id, bw.weapon_id, bw.mod_id, bw.alt_id,
                      w.name AS weapon_name, wm.name AS mod_name
               FROM background_weapons bw
               JOIN weapons w ON w.id = bw.weapon_id
               LEFT JOIN weapon_mods wm ON wm.id = bw.mod_id
               WHERE bw.background_id = ?"#,
            background_id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();

    let weapons: Vec<WeaponRow> = weapon_rows.into_iter().map(|r| WeaponRow {
        id: r.id, bg_id: r.background_id.unwrap_or_default(),
        weapon_id: r.weapon_id.unwrap_or_default(),
        weapon_name: r.weapon_name.unwrap_or_default(),
        mod_id: r.mod_id,
        mod_name: r.mod_name,
        alt_id: r.alt_id,
    }).collect();

    // Ammo
    let ammo_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT ba.ammo_id, ba.quantity, ba.bg_weapon_id, a.name AS ammo_name
               FROM background_ammo ba
               JOIN ammo a ON a.id = ba.ammo_id
               WHERE ba.bg_weapon_id IN (
                   SELECT id FROM background_weapons WHERE background_id = ?
               )"#,
            background_id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();

    let ammo: Vec<AmmoRow> = ammo_rows.into_iter().map(|r| AmmoRow {
        ammo_id: r.ammo_id.unwrap_or_default(),
        ammo_name: r.ammo_name.unwrap_or_default(),
        quantity: r.quantity.unwrap_or_default(),
        bg_weapon_id: r.bg_weapon_id.unwrap_or_default(),
    }).collect();

    // Apparel
    let apparel_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT ba.id, ba.background_id, ba.apparel_id, ba.alt_id,
                      a.name AS apparel_name
               FROM background_apparel ba
               JOIN apparel a ON a.id = ba.apparel_id
               WHERE ba.background_id = ?"#,
            background_id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();

    let apparel: Vec<ApparelRow> = apparel_rows.into_iter().map(|r| ApparelRow {
        id: r.id, bg_id: r.background_id.unwrap_or_default(),
        apparel_id: r.apparel_id.unwrap_or_default(),
        apparel_name: r.apparel_name.unwrap_or_default(),
        alt_id: r.alt_id,
    }).collect();

    // Consumables
    let consumable_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bc.id, bc.background_id, bc.consumable_id, bc.alt_id,
                      c.name AS consumable_name
               FROM background_consumables bc
               JOIN consumables c ON c.id = bc.consumable_id
               WHERE bc.background_id = ?"#,
            background_id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();

    let consumables: Vec<ConsumableRow> = consumable_rows.into_iter().map(|r| ConsumableRow {
        id: r.id, bg_id: r.background_id.unwrap_or_default(),
        consumable_id: r.consumable_id.unwrap_or_default(),
        consumable_name: r.consumable_name.unwrap_or_default(),
        alt_id: r.alt_id,
    }).collect();

    // Robot modules
    let module_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT brm.id, brm.background_id, brm.robot_module_id, brm.alt_id,
                      rm.name AS module_name
               FROM background_robot_modules brm
               JOIN robot_modules rm ON rm.id = brm.robot_module_id
               WHERE brm.background_id = ?"#,
            background_id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();

    let robot_modules: Vec<RobotModuleRow> = module_rows.into_iter().map(|r| RobotModuleRow {
        id: r.id, bg_id: r.background_id.unwrap_or_default(),
        module_id: r.robot_module_id.unwrap_or_default(),
        module_name: r.module_name.unwrap_or_default(),
        alt_id: r.alt_id,
    }).collect();

    // Gear
    let gear_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bg.gear_id, g.name AS gear_name
               FROM background_gear bg
               JOIN gear g ON g.id = bg.gear_id
               WHERE bg.background_id = ?"#,
            background_id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();

    let gear: Vec<GearRow> = gear_rows.into_iter().map(|r| GearRow {
        gear_id: r.gear_id.unwrap_or_default(),
        gear_name: r.gear_name.unwrap_or_default(),
    }).collect();

    let misc = bg.misc.trim_matches(|c| c == '{' || c == '}' || c == '"')
        .split("\",\"").collect::<Vec<_>>().join(", ");

    ResolvedBackground {
        id: bg.id, name: bg.name,
        weapon_slots: resolve_weapon_slots(weapons),
        apparel_slots: resolve_apparel_slots(apparel),
        consumable_slots: resolve_consumable_slots(consumables),
        robot_module_slots: resolve_robot_module_slots(robot_modules),
        ammo, gear,
        caps: bg.caps, misc,
        trinket: bg.trinket, food: bg.food, forage: bg.forage,
        bev: bg.bev, chem: bg.chem, ammo_count: bg.ammo,
        aid: bg.aid, odd: bg.odd, outcast: bg.outcast, junk: bg.junk,
    }
}

// ── UI Selection State ────────────────────────────────────────────────────────

/// Per-slot selection — index into the options vec, or None if unresolved
#[derive(Debug, Clone)]
pub enum SlotSelection {
    Fixed,                          // nothing to select
    Chosen(usize),                  // index into Choice vec
    ManyForOneChosen(usize),        // index (0=take the one, 1=take the many)
    SingleOrDoubleChosen {
        take_single: bool,
        double_picks: Vec<Option<usize>>, // one per double_choices slot
    },
    SingleOrPackChosen(bool),       // true=take single, false=take pack
}

pub struct EquipmentState {
    pub all_backgrounds: Vec<BackgroundRow>,
    pub origin_id: Option<i64>,
    pub selected_bg_idx: Option<usize>,
    pub current_bg: Option<ResolvedBackground>,

    pub weapon_selections: Vec<SlotSelection>,
    pub apparel_selections: Vec<SlotSelection>,
    pub consumable_selections: Vec<SlotSelection>,
    pub robot_module_selections: Vec<SlotSelection>,
}

impl EquipmentState {
    pub fn new(all_backgrounds: Vec<BackgroundRow>) -> Self {
        Self {
            all_backgrounds,
            origin_id: None,
            selected_bg_idx: None,
            current_bg: None,
            weapon_selections: vec![],
            apparel_selections: vec![],
            consumable_selections: vec![],
            robot_module_selections: vec![],
        }
    }

    /// Only backgrounds matching the selected origin
    pub fn available_backgrounds(&self) -> Vec<(usize, &BackgroundRow)> {
        self.all_backgrounds.iter()
            .enumerate()
            .filter(|(_, bg)| {
                self.origin_id.map(|oid| bg.origin_id == oid).unwrap_or(true)
            })
            .collect()
    }

    fn reset_selection(&mut self) {
        self.selected_bg_idx = None;
        self.current_bg = None;
        self.weapon_selections.clear();
        self.apparel_selections.clear();
        self.consumable_selections.clear();
        self.robot_module_selections.clear();
    }

    pub fn load_background(&mut self, db: &Db, idx: usize) {
        let id = self.all_backgrounds[idx].id;
        self.selected_bg_idx = Some(idx);
        let bg = load_background_equipment(db, id);
        self.weapon_selections = default_selections(&bg.weapon_slots);
        self.apparel_selections = default_apparel_selections(&bg.apparel_slots);
        self.consumable_selections = default_selections_generic(bg.consumable_slots.len());
        self.robot_module_selections = default_selections_generic(bg.robot_module_slots.len());
        self.current_bg = Some(bg);
    }

    pub fn is_complete(&self) -> bool {
        let Some(bg) = &self.current_bg else { return false; };
        self.selected_bg_idx.is_some()
            && selections_complete(&self.weapon_selections, bg.weapon_slots.len())
            && selections_complete(&self.apparel_selections, bg.apparel_slots.len())
            && selections_complete(&self.consumable_selections, bg.consumable_slots.len())
            && selections_complete(&self.robot_module_selections, bg.robot_module_slots.len())
    }
}

fn default_selections<T>(slots: &[T]) -> Vec<SlotSelection>
where T: IsFixed,
{
    slots.iter().map(|s| {
        if s.is_fixed() { SlotSelection::Fixed } else { SlotSelection::Chosen(usize::MAX) }
    }).collect()
}

fn default_selections_generic(len: usize) -> Vec<SlotSelection> {
    vec![SlotSelection::Chosen(usize::MAX); len]
}

fn default_apparel_selections(slots: &[ApparelSlot]) -> Vec<SlotSelection> {
    slots.iter().map(|s| match s {
        ApparelSlot::Fixed(_) => SlotSelection::Fixed,
        ApparelSlot::Choice(_) => SlotSelection::Chosen(usize::MAX),
        ApparelSlot::SingleOrDouble { double_choices, .. } =>
            SlotSelection::SingleOrDoubleChosen {
                take_single: true,
                double_picks: vec![None; double_choices.len()],
            },
        ApparelSlot::SingleOrPack { .. } => SlotSelection::SingleOrPackChosen(true),
    }).collect()
}

fn selections_complete(sels: &[SlotSelection], _len: usize) -> bool {
    sels.iter().all(|s| match s {
        SlotSelection::Fixed => true,
        SlotSelection::Chosen(i) => *i != usize::MAX,
        SlotSelection::ManyForOneChosen(i) => *i != usize::MAX,
        SlotSelection::SingleOrDoubleChosen { take_single, double_picks } =>
            *take_single || double_picks.iter().all(|p| p.is_some()),
        SlotSelection::SingleOrPackChosen(_) => true,
    })
}

trait IsFixed { fn is_fixed(&self) -> bool; }
impl IsFixed for WeaponSlot {
    fn is_fixed(&self) -> bool { matches!(self, WeaponSlot::Fixed(_)) }
}
impl IsFixed for ConsumableSlot {
    fn is_fixed(&self) -> bool { matches!(self, ConsumableSlot::Fixed(_)) }
}
impl IsFixed for RobotModuleSlot {
    fn is_fixed(&self) -> bool { matches!(self, RobotModuleSlot::Fixed(_)) }
}

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render_equipment(
    ui: &Ui,
    window: &Window,
    state: &mut EquipmentState,
    db: &Db,
    screen: &mut AppScreen,
) {
    let (win_w, win_h) = window.size();
    let w = (win_w as f32 * 0.65).min(960.0);
    let h = win_h as f32 * 0.85;
    let bar_h = crate::BAR_HEIGHT;
    let content_h = win_h as f32 - bar_h;

    let Some(_tok) = ui.window("##equipment")
        .title_bar(false).resizable(false).movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, bar_h + (content_h - h) * 0.5],
            imgui::Condition::Always,
        )
        .begin()
    else { return; };

    ui.text("EQUIPMENT");
    ui.separator();
    ui.spacing();

    // Background selector
    ui.text("Background:");
    ui.same_line();
    ui.set_next_item_width(280.0);
    let preview = state.selected_bg_idx
        .map(|i| state.all_backgrounds[i].name.as_str())
        .unwrap_or("Select background...");
    // Collect just what we need — ends the immutable borrow immediately
    let bg_names: Vec<(usize, String)> = state.available_backgrounds()
        .into_iter()
        .map(|(i, bg)| (i, bg.name.clone()))
        .collect();
    let preview = state.selected_bg_idx
        .and_then(|i| state.all_backgrounds.get(i))
        .map(|bg| bg.name_as_str())
        .unwrap_or("Select background");

    ui.text("Background:");
    ui.same_line();
    ui.set_next_item_width(280.0);
    if let Some(_cb) = ui.begin_combo("##bg_select", preview) {
        for (i, name) in &bg_names {
            let sel = state.selected_bg_idx == Some(*i);
            if ui.selectable_config(name.as_str()).selected(sel).build() {
                if state.selected_bg_idx != Some(*i) {
                    state.load_background(db, *i);
                }
            }
        }
    }
    /*
    if let Some(_cb) = ui.begin_combo("##bg_select", preview) {
        for (i, name) in &bg_names {
            let sel = state.selected_bg_idx == Some(*i);
            if ui.selectable_config(name.as_str()).selected(sel).build() {
                if state.selected_bg_idx != Some(*i) {
                    state.load_background(db, *i);
                }
            }
        }
    }
    */

    ui.spacing();

    let Some(bg) = &state.current_bg else {
        ui.text_disabled("Select a background to see starting equipment.");
        render_equipment_footer(ui, h, screen, false);
        return;
    };

    // Clone what we need to avoid borrow issues during render
    let bg = bg.clone();
    let list_h = h - 100.0;
    let Some(_child) = ui.child_window("##eq_scroll")
        .size([w - 16.0, list_h])
        .begin()
    else { return; };

    // ── Weapons ───────────────────────────────────────────────────
    if !bg.weapon_slots.is_empty() {
        ui.text("WEAPONS");
        ui.separator();
        ui.spacing();

        for (i, slot) in bg.weapon_slots.iter().enumerate() {
            render_weapon_slot(ui, i, slot, &mut state.weapon_selections[i], &bg.ammo);
            ui.spacing();
        }
        ui.spacing();
    }

    // ── Apparel ───────────────────────────────────────────────────
    if !bg.apparel_slots.is_empty() {
        ui.text("APPAREL");
        ui.separator();
        ui.spacing();

        for (i, slot) in bg.apparel_slots.iter().enumerate() {
            render_apparel_slot(ui, i, slot, &mut state.apparel_selections[i]);
            ui.spacing();
        }
        ui.spacing();
    }

    // ── Consumables ───────────────────────────────────────────────
    if !bg.consumable_slots.is_empty() {
        ui.text("CONSUMABLES");
        ui.separator();
        ui.spacing();

        for (i, slot) in bg.consumable_slots.iter().enumerate() {
            render_consumable_slot(ui, i, slot, &mut state.consumable_selections[i]);
            ui.spacing();
        }
        ui.spacing();
    }

    // ── Robot Modules ─────────────────────────────────────────────
    if !bg.robot_module_slots.is_empty() {
        ui.text("ROBOT MODULES");
        ui.separator();
        ui.spacing();

        for (i, slot) in bg.robot_module_slots.iter().enumerate() {
            render_robot_module_slot(ui, i, slot, &mut state.robot_module_selections[i]);
            ui.spacing();
        }
        ui.spacing();
    }

    // ── Gear ──────────────────────────────────────────────────────
    if !bg.gear.is_empty() {
        ui.text("GEAR");
        ui.separator();
        ui.spacing();
        for g in &bg.gear {
            ui.text(format!("  {}", g.gear_name));
        }
        ui.spacing();
    }

    // ── Misc ──────────────────────────────────────────────────────
    ui.text("MISC");
    ui.separator();
    ui.spacing();
    ui.text(format!("  Caps: {}", bg.caps));
    if !bg.misc.is_empty()   { ui.text(format!("  Misc: {}", bg.misc)); }
    if bg.trinket > 0  { ui.text(format!("  Trinket x{}", bg.trinket)); }
    if bg.food > 0     { ui.text(format!("  Food x{}", bg.food)); }
    if bg.forage > 0   { ui.text(format!("  Forage x{}", bg.forage)); }
    if bg.bev > 0      { ui.text(format!("  Beverages x{}", bg.bev)); }
    if bg.chem > 0     { ui.text(format!("  Chems x{}", bg.chem)); }
    if bg.ammo_count > 0  { ui.text(format!("  Ammo x{}", bg.ammo_count)); }
    if bg.aid > 0      { ui.text(format!("  Aid x{}", bg.aid)); }
    if bg.odd > 0      { ui.text(format!("  Oddities x{}", bg.odd)); }
    if bg.outcast > 0  { ui.text(format!("  Outcast Equipment x{}", bg.outcast)); }
    if bg.junk > 0     { ui.text(format!("  Junk x{}", bg.junk)); }

    drop(_child);

    render_equipment_footer(ui, h, screen, state.is_complete());
}

// ── Slot renderers ────────────────────────────────────────────────────────────

fn render_weapon_slot(
    ui: &Ui,
    idx: usize,
    slot: &WeaponSlot,
    sel: &mut SlotSelection,
    ammo: &[AmmoRow],
) {
    match slot {
        WeaponSlot::Fixed(opt) => {
            render_weapon_option_label(ui, opt);
            render_ammo_for(ui, opt.bg_weapon_id, ammo);
        }
        WeaponSlot::Choice(options) => {
            let chosen_idx = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_idx < options.len() {
                weapon_label(&options[chosen_idx])
            } else {
                format!("Weapon {} — choose...", idx + 1)
            };
            ui.set_next_item_width(300.0);
            if let Some(_cb) = ui.begin_combo(format!("##wslot_{}", idx), &preview) {
                for (oi, opt) in options.iter().enumerate() {
                    let s = chosen_idx == oi;
                    if ui.selectable_config(&weapon_label(opt)).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
            if chosen_idx < options.len() {
                render_ammo_for(ui, options[chosen_idx].bg_weapon_id, ammo);
            }
        }
        WeaponSlot::ManyForOne(give_up, get_one) => {
            let chosen = if let SlotSelection::ManyForOneChosen(i) = sel { *i } else { 0 };
            ui.text(format!("Choose: take {} OR give up all for {}",
                weapon_label(get_one),
                give_up.iter().map(|w| weapon_label(w)).collect::<Vec<_>>().join(" + "),
            ));
            let mut take_one = chosen == 0;
            if ui.radio_button_bool(format!("Take {}##mfo_one_{}", weapon_label(get_one), idx), take_one) {
                *sel = SlotSelection::ManyForOneChosen(0);
            }
            ui.same_line();
            if ui.radio_button_bool(
                format!("Give up for {}##mfo_many_{}", give_up.iter().map(|w| weapon_label(w)).collect::<Vec<_>>().join("+"), idx),
                !take_one
            ) {
                *sel = SlotSelection::ManyForOneChosen(1);
            }
        }
    }
}

fn render_apparel_slot(ui: &Ui, idx: usize, slot: &ApparelSlot, sel: &mut SlotSelection) {
    match slot {
        ApparelSlot::Fixed(opt) => {
            ui.text(format!("  {}", opt.name));
        }
        ApparelSlot::Choice(options) => {
            let chosen_idx = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_idx < options.len() {
                options[chosen_idx].name.clone()
            } else {
                format!("Apparel {} — choose...", idx + 1)
            };
            ui.set_next_item_width(300.0);
            if let Some(_cb) = ui.begin_combo(format!("##aslot_{}", idx), &preview) {
                for (oi, opt) in options.iter().enumerate() {
                    let s = chosen_idx == oi;
                    if ui.selectable_config(&opt.name).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
        }
        ApparelSlot::SingleOrDouble { single, double_choices } => {
            let (take_single, double_picks) = if let SlotSelection::SingleOrDoubleChosen {
                take_single, double_picks
            } = sel {
                (take_single, double_picks)
            } else { return; };

            if ui.radio_button_bool(format!("Take {}##sd_single_{}", single.name, idx), *take_single) {
                *take_single = true;
            }
            ui.same_line();
            if ui.radio_button_bool(format!("Take two pieces##sd_double_{}", idx), !*take_single) {
                *take_single = false;
            }
            if !*take_single {
                for (di, choices) in double_choices.iter().enumerate() {
                    let picked = double_picks[di];
                    let preview = picked
                        .map(|i| choices[i].name.clone())
                        .unwrap_or_else(|| format!("Slot {} — choose...", di + 1));
                    ui.set_next_item_width(280.0);
                    if let Some(_cb) = ui.begin_combo(format!("##adbl_{}_{}", idx, di), &preview) {
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
        ApparelSlot::SingleOrPack { single, pack } => {
            let take_single = if let SlotSelection::SingleOrPackChosen(b) = sel { b } else { return; };
            if ui.radio_button_bool(format!("Take {}##sp_single_{}", single.name, idx), *take_single) {
                *take_single = true;
            }
            ui.same_line();
            let pack_label = pack.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(" + ");
            if ui.radio_button_bool(format!("Take pack: {}##sp_pack_{}", pack_label, idx), !*take_single) {
                *take_single = false;
            }
        }
    }
}

fn render_consumable_slot(ui: &Ui, idx: usize, slot: &ConsumableSlot, sel: &mut SlotSelection) {
    match slot {
        ConsumableSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        ConsumableSlot::Choice(options) => {
            let chosen_idx = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_idx < options.len() {
                options[chosen_idx].name.clone()
            } else {
                format!("Consumable {} — choose...", idx + 1)
            };
            ui.set_next_item_width(280.0);
            if let Some(_cb) = ui.begin_combo(format!("##cslot_{}", idx), &preview) {
                for (oi, opt) in options.iter().enumerate() {
                    let s = chosen_idx == oi;
                    if ui.selectable_config(&opt.name).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
        }
        ConsumableSlot::ManyForOne(give_up, get_one) => {
            let chosen = if let SlotSelection::ManyForOneChosen(i) = sel { *i } else { 0 };
            ui.text(format!("Choose: {} OR give up for {}",
                get_one.name,
                give_up.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(" + "),
            ));
            if ui.radio_button_bool(format!("Take {}##cmfo_one_{}", get_one.name, idx), chosen == 0) {
                *sel = SlotSelection::ManyForOneChosen(0);
            }
            ui.same_line();
            if ui.radio_button_bool(format!("Take all##cmfo_many_{}", idx), chosen == 1) {
                *sel = SlotSelection::ManyForOneChosen(1);
            }
        }
    }
}

fn render_robot_module_slot(ui: &Ui, idx: usize, slot: &RobotModuleSlot, sel: &mut SlotSelection) {
    match slot {
        RobotModuleSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        RobotModuleSlot::Choice(options) => {
            let chosen_idx = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_idx < options.len() {
                options[chosen_idx].name.clone()
            } else {
                format!("Module {} — choose...", idx + 1)
            };
            ui.set_next_item_width(280.0);
            if let Some(_cb) = ui.begin_combo(format!("##rmslot_{}", idx), &preview) {
                for (oi, opt) in options.iter().enumerate() {
                    let s = chosen_idx == oi;
                    if ui.selectable_config(&opt.name).selected(s).build() {
                        *sel = SlotSelection::Chosen(oi);
                    }
                }
            }
        }
    }
}

// ── Label helpers ─────────────────────────────────────────────────────────────

fn weapon_label(opt: &WeaponOption) -> String {
    let mut s = opt.name.clone();
    if let Some(m) = &opt.mod_name { s.push_str(&format!(" w/ {}", m)); }
    if !opt.extra_mods.is_empty() {
        s.push_str(&format!(" + {}", opt.extra_mods.join(", ")));
    }
    s
}

fn render_weapon_option_label(ui: &Ui, opt: &WeaponOption) {
    ui.text(format!("  {}", weapon_label(opt)));
}

fn render_ammo_for(ui: &Ui, bg_weapon_id: i64, ammo: &[AmmoRow]) {
    for a in ammo.iter().filter(|a| a.bg_weapon_id == bg_weapon_id) {
        ui.text_colored([0.6, 0.6, 0.6, 1.0],
            format!("    Ammo: {} ({})", a.ammo_name, a.quantity));
    }
}

fn render_equipment_footer(ui: &Ui, win_h: f32, screen: &mut AppScreen, complete: bool) {
    let footer_y = win_h - 44.0;
    ui.set_cursor_pos([16.0, footer_y]);
    if ui.button("< Back") { *screen = AppScreen::Stats; }
    ui.same_line();
    let _g = (!complete).then(|| ui.begin_disabled(true));
    if ui.button("Review >") {
        //*screen = AppScreen::Review;
        //render_placeholder(ui, window, "review", screen);
        }
    drop(_g);
    if !complete {
        ui.same_line();
        ui.text_disabled("Make all equipment choices to continue.");
    }
}