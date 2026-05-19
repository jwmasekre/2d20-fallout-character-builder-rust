use std::collections::{HashMap, HashSet};

use imgui::Ui;

use crate::{db::{AmmoRow, ApparelRow, ConsumableRow, GearRow, RobotModuleRow, WeaponRow}, structs::AppConfig};

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

pub trait IsFixed { fn is_fixed(&self) -> bool; }
impl IsFixed for WeaponSelSlot {
    fn is_fixed(&self) -> bool { matches!(self, WeaponSelSlot::Fixed(_)) }
}
impl IsFixed for ConsumableSelSlot {
    fn is_fixed(&self) -> bool { matches!(self, ConsumableSelSlot::Fixed(_)) }
}
impl IsFixed for RobotModuleSelSlot {
    fn is_fixed(&self) -> bool { matches!(self, RobotModuleSelSlot::Fixed(_)) }
}

pub fn default_selections<T>(slots: &[T]) -> Vec<SlotSelection>
where T: IsFixed,
{
    slots.iter().map(|s| {
        if s.is_fixed() { SlotSelection::Fixed } else { SlotSelection::Chosen(usize::MAX) }
    }).collect()
}
pub fn default_apparel_selections(slots: &[ApparelSelSlot]) -> Vec<SlotSelection> {
    slots.iter().map(|s| match s {
        ApparelSelSlot::Fixed(_) => SlotSelection::Fixed,
        ApparelSelSlot::Choice(_) => SlotSelection::Chosen(usize::MAX),
        ApparelSelSlot::SingleOrDouble(_,double_choices) => SlotSelection::SingleOrDoubleChosen(true,vec![None; double_choices.len()],),
        ApparelSelSlot::SingleOrPack(..) => SlotSelection::SingleOrPackChosen(true),
    }).collect()
}

//we may want to just feed a full-blown weapon, rather than this option, for ease of implementation into the character. same with the apparel.
pub fn weapon_option(row: &WeaponRow) -> WeaponOption {
    WeaponOption {
        bg_weapon_id: row.id,
        weapon_id: row.weapon_id,
        name: row.weapon_name.clone(),
        mod_name: row.mod_name.clone(),
        extra_mods: vec![],
    }
}
pub fn apparel_option(row: &ApparelRow) -> ApparelOption {
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

pub fn weapon_label(opt: &WeaponOption) -> String {
    let mut s = opt.name.clone();
    if let Some(m) = &opt.mod_name { s.push_str(&format!(" w/ {}", m)); }
    if !opt.extra_mods.is_empty() {
        s.push_str(&format!(" + {}", opt.extra_mods.join(", ")));
    }
    s
}

pub fn render_weapon_option_label(ui: &Ui, opt: &WeaponOption) {
    ui.text(format!("  {}", weapon_label(opt)))
}

pub fn render_ammo_for(ui: &Ui, bg_weapon_id: i32, ammo: &[AmmoRow]) {
    for a in ammo.iter().filter(|a| a.bg_weapon_id == bg_weapon_id) {
        ui.text_disabled(format!("    Ammo: {} ({})", a.ammo_name, a.quantity));
    }
}

pub fn render_weapon_slot(ui: &Ui, index: usize, slot: &WeaponSelSlot, sel: &mut SlotSelection, ammo: &[AmmoRow], cfg: &AppConfig) -> bool {
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
            ui.set_next_item_width(300.0 * cfg.ui_scale);
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

pub fn render_apparel_slot(ui: &Ui, index: usize, slot: &ApparelSelSlot, sel: &mut SlotSelection, cfg: &AppConfig) -> bool {
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
            ui.set_next_item_width(300.0 * cfg.ui_scale);
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
                    ui.set_next_item_width(280.0 * cfg.ui_scale);
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

pub fn render_consumable_slot(ui: &Ui, index: usize, slot: &ConsumableSelSlot, sel: &mut SlotSelection, cfg: &AppConfig) -> bool {
    match slot {
        ConsumableSelSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        ConsumableSelSlot::Choice(opts) => {
            let chosen_index = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_index < opts.len() {
                opts[chosen_index].name.clone()
            } else {
                format!("Consumable {} - choose...", index + 1)
            };
            ui.set_next_item_width(280.0 * cfg.ui_scale);
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

pub fn render_robot_module_slot(ui: &Ui, index: usize, slot: &RobotModuleSelSlot, sel: &mut SlotSelection, cfg: &AppConfig) -> bool {
    match slot {
        RobotModuleSelSlot::Fixed(opt) => { ui.text(format!("  {}", opt.name)); }
        RobotModuleSelSlot::Choice(opts) => {
            let chosen_index = if let SlotSelection::Chosen(i) = sel { *i } else { usize::MAX };
            let preview = if chosen_index < opts.len() {
                opts[chosen_index].name.clone()
            } else {
                format!("Module {} - choose...", index + 1)
            };
            ui.set_next_item_width(280.0 * cfg.ui_scale);
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