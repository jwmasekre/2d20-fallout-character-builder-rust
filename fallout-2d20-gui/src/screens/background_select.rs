use crate::{character::Character, db::Db, theme::render_window};
use std::collections::{HashMap, HashSet};
use imgui::Ui;
use sdl2::video::Window;

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

/*
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
    pub fn qual_not_eff(&self, name: &str) -> bool {
        self.quality_names.contains(&name.to_lowercase())
    }
}
*/

pub struct BackgroundState {
    pub all_backgrounds: Vec<BackgroundRow>,
    pub selected_index: Option<usize>,
    pub current_background: Option<ResolvedBackground>,
    pub weapon_selections: Vec<SlotSelection>,
    pub apparel_selections: Vec<SlotSelection>,
    pub consumable_selections: Vec<SlotSelection>,
    pub robot_module_selections: Vec<SlotSelection>,
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
        }
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
    pub fn is_complete(&self) -> bool {
        self.selected_index.is_some() && selections_complete(&self.weapon_selections) && selections_complete(&self.apparel_selections) && selections_complete(&self.consumable_selections) && selections_complete(&self.robot_module_selections)
    }
}

fn selections_complete(sels: &[SlotSelection]) -> bool {
    sels.iter().all(|s| match s {
        SlotSelection::Fixed => true,
        SlotSelection::Chosen(i) => *i != usize::MAX,
        SlotSelection::ManyForOneChosen(i) => *i != usize::MAX,
        SlotSelection::SingleOrDoubleChosen { take_single, double_picks } => *take_single || double_picks.iter().all(|p| p.is_some()),
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
pub enum WeaponSlot {
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
pub enum ApparelSlot {
    Fixed(ApparelOption),
    Choice(Vec<ApparelOption>),
    SingleOrDouble {
        single: ApparelOption,
        double_choices: Vec<Vec<ApparelOption>>,
    },
    SingleOrPack {
        single: ApparelOption,
        pack: Vec<ApparelOption>,
    },
}

#[derive(Debug, Clone)]
pub struct ConsumableOption {
    pub bg_consumable_id: i32,
    pub consumable_id: i32,
    pub name: String,
    pub wgt: i32,
}

#[derive(Debug, Clone)]
pub enum ConsumableSlot {
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
pub enum RobotModuleSlot {
    Fixed(RobotModuleOption),
    Choice(Vec<RobotModuleOption>),
}

#[derive(Debug, Clone)]
pub struct ResolvedBackground {
    pub id: i32,
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

//used to handle selection of options, covers all the possible selection types
#[derive(Debug, Clone)]
pub enum SlotSelection {
    Fixed,
    Chosen(usize),
    ManyForOneChosen(usize),
    SingleOrDoubleChosen {
        take_single: bool,
        double_picks: Vec<Option<usize>>,
    },
    SingleOrPackChosen(bool),
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

fn default_selections<T>(slots: &[T]) -> Vec<SlotSelection>
where T: IsFixed,
{
    slots.iter().map(|s| {
        if s.is_fixed() { SlotSelection::Fixed } else { SlotSelection::Chosen(usize::MAX) }
    }).collect()
}
fn default_apparel_selections(slots: &[ApparelSlot]) -> Vec<SlotSelection> {
    slots.iter().map(|s| match s {
        ApparelSlot::Fixed(_) => SlotSelection::Fixed,
        ApparelSlot::Choice(_) => SlotSelection::Chosen(usize::MAX),
        ApparelSlot::SingleOrDouble { double_choices, .. } => SlotSelection::SingleOrDoubleChosen { take_single: true, double_picks: vec![None; double_choices.len()], },
        ApparelSlot::SingleOrPack { .. } => SlotSelection::SingleOrPackChosen(true),
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
pub fn resolve_weapon_slots(rows: Vec<WeaponRow>) -> Vec<WeaponSlot> {
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
    let mut slots: Vec<WeaponSlot> = vec![];
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
                    slots.push(WeaponSlot::Fixed(opt));
                } else {
                    visited.insert(row.id);
                    slots.push(WeaponSlot::Fixed(weapon_option(row)));
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
            slots.push(WeaponSlot::ManyForOne(give_up, get))
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
                slots.push(WeaponSlot::Fixed(options.into_iter().next().unwrap()));
            } else {
                slots.push(WeaponSlot::Choice(options));
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

fn render_weapon_slot(ui: &Ui, index: usize, slot: &WeaponSlot, sel: &mut SlotSelection, ammo: &[AmmoRow],) {
    match slot {
        WeaponSlot::Fixed(opt) => {
            render_weapon_option_label(ui, opt);
            render_ammo_for(ui, opt.bg_weapon_id, ammo);
        }
        WeaponSlot::Choice(opts) => {
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
        WeaponSlot::ManyForOne(give_up, get_one) => {
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
}

pub fn resolve_apparel_slots(rows: Vec<ApparelRow>) -> Vec<ApparelSlot> {
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
    let mut slots: Vec<ApparelSlot> = vec![];
    //this tracks what we've already looked at (so we don't endlessly recurse, or just waste cycles reviewing stuff we've already assigned)
    let mut visited: HashSet<i32> = HashSet::new();

    //handle all the items that don't have alternates
    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) && !repeated.contains(&row.id) {
            if visited.insert(row.id) {
                slots.push(ApparelSlot::Fixed(apparel_option(row)));
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
            slots.push(ApparelSlot::Fixed(options.into_iter().next().unwrap()));
        } else {
            slots.push(ApparelSlot::Choice(options));
        }
    }

    //this is where we actually handle the weird choices
    if let Some(anchor_id) = single_anchor {
        let anchor_row = by_id[&anchor_id];
        let single_opt = apparel_option(anchor_row);
        //checking if it's either to the pick 1 of 2 twice or pick a pack
        let sibling_ids = &apparel_id_count[&anchor_row.apparel_id];
        //grab all the alt targets
        let /*mut*/ sibling_alts: Vec<i32> = sibling_ids.iter()
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
        slots.push(ApparelSlot::SingleOrDouble { single: single_opt, double_choices: deduped_choices });
    }
    //looks like we're missing the "single or pack" logic? need to do some testing
    slots
}

fn render_apparel_slot(ui: &Ui, index: usize, slot: &ApparelSlot, sel: &mut SlotSelection) {
    match slot {
        ApparelSlot::Fixed(opt) => {
            ui.text(format!("  {}", opt.name));
        }
        ApparelSlot::Choice(opts) => {
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
        ApparelSlot::SingleOrDouble { single, double_choices } => {
            let (take_single, double_picks) = if let SlotSelection::SingleOrDoubleChosen {
                take_single, double_picks
            } = sel {
                (take_single, double_picks)
            } else { return; };

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
        ApparelSlot::SingleOrPack { single, pack } => {
            let take_single = if let SlotSelection::SingleOrPackChosen(b) = sel { b } else { return; };
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
}

pub fn resolve_consumable_slots(rows: Vec<ConsumableRow>) -> Vec<ConsumableSlot> {
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

    let mut slots: Vec<ConsumableSlot> = vec![];
    let mut visited: HashSet<i32> = HashSet::new();

    for row in &rows {
        if row.alt_id.is_none() && !rev.contains_key(&row.id) {
            if visited.insert(row.id) {
                slots.push(ConsumableSlot::Fixed(ConsumableOption {
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
            slots.push(ConsumableSlot::ManyForOne(give_up, get));
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
                slots.push(ConsumableSlot::Fixed(options.into_iter().next().unwrap()));
            } else {
                slots.push(ConsumableSlot::Choice(options));
            }
        }
    }
    slots
}

fn render_consumable_slot(ui: &Ui, index: usize, slot: &ConsumableSlot, sel: &mut SlotSelection) {
    match slot {
        ConsumableSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        ConsumableSlot::Choice(opts) => {
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
        ConsumableSlot::ManyForOne(give_up, get_one) => {
            let chosen = if let SlotSelection::ManyForOneChosen(i) = sel { *i } else { 0 };
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
}

//robot module is even simpler
pub fn resolve_robot_module_slots(rows: Vec<RobotModuleRow>) -> Vec<RobotModuleSlot> {
    let mut fwd: HashMap<i32, i32> = HashMap::new();
    let mut rev: HashMap<i32, Vec<i32>> = HashMap::new();
    let by_id: HashMap<i32, &RobotModuleRow> = rows.iter().map(|r| (r.id, r)).collect();

    for row in &rows {
        if let Some(alt) = row.alt_id {
            fwd.insert(row.id, alt);
            rev.entry(alt).or_default().push(row.id);
        }
    }

    let mut slots: Vec<RobotModuleSlot> = vec![];
    let mut visited: HashSet<i32> = HashSet::new();

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
            slots.push(RobotModuleSlot::Fixed(options.into_iter().next().unwrap()));
        } else {
            slots.push(RobotModuleSlot::Choice(options));
        }
    }
    slots
}

fn render_robot_module_slot(ui: &Ui, index: usize, slot: &RobotModuleSlot, sel: &mut SlotSelection) {
    match slot {
        RobotModuleSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        RobotModuleSlot::Choice(opts) => {
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
}

pub fn render_background_select(
    ui: &Ui,
    window: &Window,
    state: &mut BackgroundState,
    db: &Db,
    character: &mut Character,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##background_select", "Background Select")
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
    let list_h = h - 100.0;
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
            render_weapon_slot(ui, i, slot, &mut state.weapon_selections[i], &bg.ammo);
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
            render_apparel_slot(ui, i, slot, &mut state.apparel_selections[i]);
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
            render_consumable_slot(ui, i, slot, &mut state.consumable_selections[i]);
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
            render_robot_module_slot(ui, i, slot, &mut state.robot_module_selections[i]);
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

    //ends the scroll window
    drop(_child);
    h
}