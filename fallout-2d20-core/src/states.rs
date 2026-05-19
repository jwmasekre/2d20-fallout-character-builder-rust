use std::path::PathBuf;
use crate::{
    build_origin_labels,
    background_slots::{
        ResolvedBackground,
        SlotSelection,
        default_apparel_selections,
        default_selections
    },
    character::{
        Character,
        AmmoData,
        AmmoInv,
        Apparel,
        ApparelType,
        CompanionType,
        Consumable,
        Gear,
        Junk,
        Origin,
        RobotModule,
        RobotType,
        TagType,
        Trait,
        Weapon,
    },
    db::{
        Db,
        BackgroundRow,
        OriginRow,
        PerkRow,
        TraitRow,
        load_background_equipment,
        load_backgrounds,
        load_ghoul_traits,
        load_origins,
        load_perks,
        load_traits,
        resolve_apparel,
        resolve_consumables,
        resolve_remaining_eq,
        resolve_robot_modules,
        resolve_weapons,
    },
    structs::{
        PerkResolution,
        PerkResolutionPopup
    },
};
use crate::constants::NULL_PARTY;

//load character
pub struct LoadCharacterState {
    pub characters: Vec<(String, String, String)>, // (id, name, player_name)
    pub loaded: bool,
    pub selected: Option<usize>,
    pub error: Option<String>,
    pub confirm_delete: Option<usize>,
}
impl LoadCharacterState {
    pub fn new() -> Self {
        Self {
            characters: vec![],
            loaded: false,
            selected: None,
            error: None,
            confirm_delete: None,
        }
    }
    pub fn reset(&mut self) {
        self.characters = vec![];
        self.loaded = false;
        self.selected = None;
        self.error = None;
    }
    pub fn load_list(&mut self, db: &Db) {
        if self.loaded { return; }
        let rows = db.block_on(async {
            sqlx::query!(
                r#"SELECT c.id, c.character_name, p.username
                   FROM characters c
                   JOIN players p ON p.id = c.player_id
                   ORDER BY p.username, c.character_name"#
            ).fetch_all(&db.pool).await
        }).unwrap_or_default();
        self.characters = rows.into_iter().map(|r| (
            r.id.unwrap_or_default(),
            r.character_name.unwrap_or_else(|| "(unnamed)".to_string()),
            r.username.unwrap_or_else(|| "(unknown player)".to_string()),
        )).collect();
        self.loaded = true;
    }
}

//import character
#[derive(Debug, Clone, PartialEq)]
pub enum ImportStep {
    Idle,
    Confirm(Character),      // file loaded, ask about overwrite
    Done,
    Error(String),
}
pub struct ImportState {
    pub step: ImportStep,
}
impl ImportState {
    pub fn new() -> Self {
        Self { step: ImportStep::Idle }
    }

    pub fn reset(&mut self) {
        self.step = ImportStep::Idle;
    }

    /// Returns true if the character id already exists in the db
    pub fn id_exists(db: &Db, id: &str) -> bool {
        db.block_on(async {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM characters WHERE id = ?", id
            ).fetch_one(&db.pool).await
        }).unwrap_or(0) > 0
    }

    /// Try to load a json file into a Character struct
    pub fn load_from_file(path: &PathBuf) -> Result<Character, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not read file: {e}"))?;
        serde_json::from_str::<Character>(&raw)
            .map_err(|e| format!("Invalid character JSON: {e}"))
    }
}

//create character
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerChoice {
    Unset,
    New,
    Existing(String), // uuid
}
#[derive(Debug, Clone, PartialEq)]
pub enum PartyChoice {
    Unset,
    None,             // joins null party
    New,
    Existing(String), // uuid
}
pub struct NewCharacterSetupState {
    // step 0 = player, step 1 = party
    pub step: usize,
    // player
    pub player_choice: PlayerChoice,
    pub new_player_name: String,
    pub players: Vec<(String, String)>, // (id, username)
    pub player_list_loaded: bool,
    // party
    pub party_choice: PartyChoice,
    pub new_party_name: String,
    pub parties: Vec<(String, String)>, // (id, name)
    pub party_list_loaded: bool,
}
impl NewCharacterSetupState {
    pub fn new() -> Self {
        Self {
            step: 0,
            player_choice: PlayerChoice::Unset,
            new_player_name: String::new(),
            players: vec![],
            player_list_loaded: false,
            party_choice: PartyChoice::Unset,
            new_party_name: String::new(),
            parties: vec![],
            party_list_loaded: false,
        }
    }
    pub fn reset(&mut self) {
        self.step = 0;
        self.player_choice = PlayerChoice::Unset;
        self.new_player_name = String::new();
        self.players = vec![];
        self.player_list_loaded = false;
        self.party_choice = PartyChoice::Unset;
        self.new_party_name = String::new();
        self.parties = vec![];
        self.party_list_loaded = false;
    }
    pub fn load_players(&mut self, db: &Db) {
        if self.player_list_loaded { return; }
        let rows = db.block_on(async {
            sqlx::query!("SELECT id, username FROM players ORDER BY username")
                .fetch_all(&db.pool).await
        }).unwrap_or_default();
        self.players = rows.into_iter()
            .map(|r| (
                r.id.unwrap_or_default(),
                r.username.unwrap_or_else(|| "(unnamed)".to_string()),
            ))
            .collect();
        self.player_list_loaded = true;
    }
    pub fn load_parties(&mut self, db: &Db) {
        if self.party_list_loaded { return; }
        let rows = db.block_on(async {
            sqlx::query!(
                r#"SELECT id, name
                FROM parties
                WHERE id != ?
                ORDER BY name
                "#,
                NULL_PARTY
            ).fetch_all(&db.pool).await
        }).unwrap_or_default();
        self.parties = rows.into_iter()
            .map(|r| (
                r.id.unwrap_or_default(),
                r.name.unwrap_or_else(|| "(unnamed)".to_string()),
            ))
            .collect();
        self.party_list_loaded = true;
    }
}

//origin
#[derive(Debug)]
pub struct OriginState {
    pub selected: bool,
    pub trait_count: i32,
    pub origin_trait_count: i32,
    pub origin_index: usize,
    pub origin_labels: Vec<String>,
    pub origin_label_to_index: Vec<Option<usize>>,
    pub origins: Vec<OriginRow>,
    pub traits: Vec<TraitRow>,
    pub _ghoul_trait: Option<TraitRow>,
}
impl OriginState {
    pub fn new(db: &Db) -> Self {
        let origins = load_origins(db);
        let (labels, label_map) = build_origin_labels(&origins);
        Self {
            selected: false,
            trait_count: 0,
            origin_trait_count: 0,
            origin_index: usize::MAX,
            origin_labels: labels,
            origin_label_to_index: label_map,
            origins,
            traits: vec![],
            _ghoul_trait: None,
        }
    }
    pub fn reset(&mut self) {
        self.selected = false;
        self.trait_count = 0;
        self.origin_trait_count = 0;
        self.origin_index = usize::MAX;
        self.traits = vec![];
        self._ghoul_trait = None;
    }
    pub fn is_complete(&self) -> bool {//do we need to be checking the character to make sure that's good to go too?
        self.selected && (self.trait_count == self.origin_trait_count)
    }
    pub fn update_origin(&mut self, character: &mut Character, background_state: &mut BackgroundState) {
        //if no origins are returned by the db, return None
        if self.origins.is_empty() { character.origin = None; return }
        //mark that the player has selected an origin
        self.selected = true;
        //capture which origin they selected
        let selected_origin = self.origins.get(self.origin_index).unwrap();
        //build the origin struct for assignment to character
        let active_origin = Origin {
            id: selected_origin.id,
            name: selected_origin.name.clone(),
            desc: selected_origin.description.clone(),
            can_ghoul: selected_origin.can_ghoul,
        };
        //grab the old origin; checking if they go from mutant -> non or vice-versa
        let old_origin = if character.origin.is_some() {character.origin.clone().unwrap().id} else {i32::MAX};
        //if they go to mutant from non, add two to str and end
        if [3,16].contains(&selected_origin.id) && ![3,16].contains(&old_origin) {
            character.special.strength.value += 2;
            character.special.endurance.value += 2;
        //if they go from mutant to non, remove two from str and end
        } else if ![3,16].contains(&selected_origin.id) && [3,16].contains(&old_origin) {
            character.special.strength.value -= 2;
            character.special.endurance.value -= 2;
        }
        //update the character origin
        character.origin = Some(active_origin);
        character.update_type();
        //update max special
        character.special.apply_max(&character.clone());
        //clear out any selected backgrounds
        background_state.reset_selection();
        //update limbs
        character.limb_dr.update_active(character.robot.clone());
        //clear out the robot hat just in case we switch to a non-robot
        if character.robot == RobotType::None { character.robot_hat.take(); }
    }
    //retrieve the traits based on the current origin
    pub fn reload_traits(&mut self, db: &Db, character: &mut Character) {
        //clear traits
        self.traits = vec![];
        character.traits = vec![];
        //return if the db doesn't return any origins
        if self.origins.is_empty() {
            return;
        }
        //grab the currently selected origin's id
        let origin_id = self.origins[self.origin_index].id;
        //retrieve traits from the db
        let traits = load_traits(db, origin_id, self);
        //retrieve the ghoul trait
        let ghoul_trait = &load_ghoul_traits(db, self)[0];
        //if the player selected the ghoul origin, mark the character as a ghoul
        if traits[0].is_ghoul_trait {
            character.ghoul = true;
        //if the player selected an origin that can't ghoul, mark them as not a ghoul
        } else if !self.origins[self.origin_index].can_ghoul {
            character.ghoul = false;
        }
        //if the character is a ghoul, set their trait to the ghoul trait and don't run any other trait logic
        if character.ghoul {
            //building the trait struct for assignment
            let active_trait= Trait {
                id: ghoul_trait.id,
                name: ghoul_trait.name.clone(),
                desc: ghoul_trait.description.clone(),
            };
            //assign the trait
            character.traits = vec![active_trait];
            //mark the number of valid traits as 1 and assigned as 1 (satisfies one of the complete conditions)
            self.trait_count = 1;
            self.origin_trait_count = 1;
        //if the character's origin only has one trait, just grab that and assign it (like the ghoul block above)
        } else if traits.len() == 1 {
            let active_trait = Trait {
                id: traits[0].id,
                name: traits[0].name.clone(),
                desc: traits[0].description.clone(),
            };
            character.traits = vec![active_trait];
            self.trait_count = 1;
            self.origin_trait_count = 1;
        //the character's origin has options, so clear out the character's traits and indicate that they've selected 0 out of a max of 2
        } else {
            character.traits = vec![];
            self.trait_count = character.traits.len() as i32;
            self.origin_trait_count = 2;
            self.traits = traits;
        }
    }
    pub fn update_trait(&self, character: &mut Character) {
        let skill_max = character.level.clamp(3,6);
        let mutant = character.is_mutant();
        let good = character.has_trait(13);
        let skills = character.skills.mut_skill_block();
        for i in 0..17 {
            if mutant {
                skills[i].max = 4.min(skill_max);
            } else {
                skills[i].max = skill_max;
            }
            if good && [0,2,3,4,5,7,8,11,12,14,15,16].contains(&i) {
                skills[i].max = 4.min(skill_max);
            }
        }
        character.calculate_xp();
    }
}

//special
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialArray {
    None,
    Balanced,   // 6,6,6,6,6,5,5
    Focused,    // 8,7,6,6,5,4,4
    Specialized,// 9,8,5,5,5,4,4
    Custom,
}
impl SpecialArray {
    //functions that look up the drop-down label...
    pub fn label(&self) -> &'static str {
        match self {
            Self::None        => "Select SPECIAL array...",
            Self::Balanced    => "Balanced    (6,6,6,6,6,5,5)",
            Self::Focused     => "Focused     (8,7,6,6,5,4,4)",
            Self::Specialized => "Specialized (9,8,5,5,5,4,4)",
            Self::Custom      => "Custom",
        }
    }
    //...and actual values of each array
    pub fn values(&self) -> Option<[i32; 7]> {
        match self {
            Self::Balanced    => Some([6,6,6,6,6,5,5]),
            Self::Focused     => Some([8,7,6,6,5,4,4]),
            Self::Specialized => Some([9,8,5,5,5,4,4]),
            _ => None,
        }
    }
}
//track validity states (no array, )
#[derive(Debug, PartialEq)]
pub struct SpecialState {
    pub selected_array: SpecialArray,
    pub assignments: [Option<i32>; 7],
    pub values: [i32; 7],
    pub can_inc: [bool; 7],
    pub can_dec: [bool; 7],
    pub gifted: bool,
    pub gifted_count: i32,
    pub trained: i32,
    pub trained_count: i32,
    pub total: i32,
}
impl SpecialState {
    pub fn new() -> Self {
        Self {
            selected_array: SpecialArray::None,
            assignments: [None; 7],
            values: [5; 7],
            can_inc: [true; 7],
            can_dec: [true; 7],
            gifted: false,
            gifted_count: 0,
            trained: 0,
            trained_count: 0,
            total: 35,
        }
    }
    pub fn reset(&mut self) {
        self.selected_array = SpecialArray::None;
        self.assignments = [None; 7];
        self.values = [5; 7];
        self.can_inc = [true; 7];
        self.can_dec = [true; 7];
        self.gifted = false;
        self.gifted_count = 0;
        self.trained = 0;
        self.trained_count = 0;
        self.total = 35;
    }
    pub fn update(&mut self, character: &Character) {
        self.total = character.special.special_block().iter().map(|s| s.value).sum();
        self.selected_array = self.selected_array;
        self.assignments = self.assignments;
        //self.values = self.values;
        //check if the character is gifted
        self.gifted = character.is_gifted();
        //count number of gifted selections
        self.gifted_count = character.special.special_block().iter().map(|s| s.gifted).filter(|&b| b).count() as i32;
        //check how much intense training the character has
        self.trained = match character.perks.iter().find(|p| p.id == 45) {
            Some(perk) => perk.ranks,
            None => 0,
        };
        //check how many times intense training has been applied
        self.trained_count = character.special.special_block().iter().map(|s| s.trained).sum();
        for (i, spec) in character.special.special_block().iter().enumerate() {
            let mut_stat = i == 0 || i == 2;
            self.can_inc[i] = spec.value < spec.max && self.remaining_points(character) > 0;
            self.can_dec[i] = spec.value > 4 + if character.is_mutant() && mut_stat { 2 } else { 0 };
        }
        //log_on_change!(self);
    }
    pub fn is_complete(&self, character: &Character) -> bool {
        (if self.gifted { self.gifted_count == 2 } else { self.gifted_count == 0 }) &&
            self.trained == self.trained_count && self.total == 40 + self.trained_count + self.gifted_count + if character.is_mutant() { 4 } else { 0 } &&
            ( self.selected_array == SpecialArray::Custom ||
                !self.assignments.iter().any(|a| a.is_none())
            )
    }
    pub fn remaining_points(&self, character: &Character) -> i32 {
        40 + if self.gifted { 2 } else { 0 } + self.trained + if character.is_mutant() { 4 } else { 0 } - self.total
    }
}

//skill
#[derive(Debug)]
pub struct SkillState {
    pub extra_trait_options: Vec<usize>,
    pub extra_tags: Vec<usize>,
    //these are exclusively if the player picks both educated and good natured
    pub x_extra_trait_options: Vec<usize>,
    pub x_extra_tags: Vec<usize>,
    pub extra_trait_count: i32,
    pub forced_trait: bool,
    pub assigned_points: i32,
    pub available_points: i32,
    pub max_assignable: i32,
    pub total_points: i32,
}
impl SkillState {
    pub fn new(character: Character) -> Self {
        let total_points = character.total_skill();
        let extra_trait_options = if character.has_any_trait(vec![1,24]) {
            vec![3,9,10]
        } else if character.has_any_trait(vec![5,11,21]) {
            vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]
        } else if character.has_trait(2) {
            vec![14]
        } else if character.has_trait(12) {
            vec![3,11]
        } else if character.has_trait(13) {
            vec![1,6,9,10,13]
        } else { vec![] };
        let x_extra_trait_options = if character.has_trait(5) && character.has_trait(13) {
            vec![1,6,9,10,13]
        } else {vec![]};
        let extra_trait_count = if character.has_trait(5) && character.has_trait(13) { 3 } else if character.has_any_trait(vec![1, 2, 5, 11, 12, 21, 24]) { 1 } else if character.has_trait(13) { 2 } else { 0 };
        let forced_trait = character.has_trait(2);
        let assigned_points = character.total_skill_ranks();
        let max_assignable = 9 + character.special.intelligence.value + character.level - 1;
        let available_points = max_assignable - assigned_points;
        Self {
            extra_trait_options,
            extra_tags: vec![],
            x_extra_trait_options,
            x_extra_tags: vec![],
            extra_trait_count,
            forced_trait,
            assigned_points,
            available_points,
            max_assignable,
            total_points,
        }
    }
    pub fn reset(&mut self, character: &mut Character) {
        self.total_points = character.total_skill();
        self.extra_trait_options = if character.has_any_trait(vec![1,24]) {
            vec![3,9,10]
        } else if character.has_any_trait(vec![5,11,21]) {
            vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]
        } else if character.has_trait(2) {
            vec![14]
        } else if character.has_trait(12) {
            vec![3,11]
        } else if character.has_trait(13) {
            vec![1,6,9,10,13]
        } else { vec![] };
        self.x_extra_trait_options = if character.has_trait(5) && character.has_trait(13) {
            vec![1,6,9,10,13]
        } else {vec![]};
        self.extra_trait_count = if character.has_trait(5) && character.has_trait(13) { 3 } else if character.has_any_trait(vec![1, 2, 5, 11, 12, 21, 24]) { 1 } else if character.has_trait(13) { 2 } else { 0 };
        self.forced_trait = character.has_trait(2);
        self.assigned_points = character.total_skill_ranks();
        self.max_assignable = 9 + character.special.intelligence.value + character.level - 1;
        self.available_points = self.max_assignable - self.assigned_points;
        self.extra_tags = vec![];
        self.x_extra_tags = vec![];
        for skill in character.skills.mut_skill_block() {
            if skill.tagged == TagType::Trait {skill.tagged = TagType::None}
        }
    }
    pub fn update(&mut self, character: &Character) {
        self.total_points = character.total_skill();
        self.extra_trait_options = if character.has_any_trait(vec![1,24]) {
            vec![3,9,10]
        } else if character.has_any_trait(vec![5,11,21]) {
            vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]
        } else if character.has_trait(2) {
            vec![14]
        } else if character.has_trait(12) {
            vec![3,11]
        } else if character.has_trait(13) {
            vec![1,6,9,10,13]
        } else { vec![] };
        self.x_extra_trait_options = if character.has_trait(5) && character.has_trait(13) {
            vec![1,6,9,10,13]
        } else {vec![]};
        self.extra_trait_count = if character.has_trait(5) && character.has_trait(13) { 3 } else if character.has_any_trait(vec![1, 2, 5, 11, 12, 21, 24]) { 1 } else if character.has_trait(13) { 2 } else { 0 };
        if !(character.has_trait(5) && character.has_trait(13)) {
            self.x_extra_tags = vec![];
        }
        self.forced_trait = character.has_trait(2);
        self.assigned_points = character.total_skill_ranks();
        self.max_assignable = 9 + character.special.intelligence.value + character.level - 1;
        self.available_points = self.max_assignable - self.assigned_points;
    }
    pub fn is_complete(&self, character: &Character) -> bool {
        let std_tag_count = character.skills.standard_tags();
        self.available_points == 0 && (self.extra_tags.len() + self.x_extra_tags.len()) as i32 == self.extra_trait_count && std_tag_count == 3
    }
}

//perk
#[derive(Debug)]
pub struct PerkState {
    pub perks: Vec<PerkRow>,
    pub taken_count: i32,
    pub perk_lim: i32,
    pub show_eligible_only: bool,
    pub show_taken: bool,
    pub show_taken_only: bool,
    pub show_flagged_only: bool,
    pub filters: [bool; 8],
    pub pending_resolution: Option<(i32, bool, String)>,
}
impl PerkState {
    pub fn new(db: &Db, character: &Character) -> Self {
        let perks = load_perks(db);
        let taken_count = character.perks.iter().map(|p| p.ranks).sum();
        let perk_lim = character.level + if character.has_trait(10) { 1 } else { 0 };
        Self {
            perks,
            taken_count,
            perk_lim,
            show_eligible_only: false,
            show_taken: true,
            show_taken_only: false,
            show_flagged_only: false,
            filters: [true; 8],
            pending_resolution: None,
        }
    }
    pub fn reset(&mut self, character: &Character) {
        let taken_count: i32 = character.perks.iter().map(|p| p.ranks).sum();
        let perk_lim = character.level + if character.has_trait(10) { 1 } else { 0 };
        self.taken_count = taken_count;
        self.perk_lim = perk_lim;
        self.show_eligible_only = false;
        self.show_taken = true;
        self.show_taken_only = false;
        self.show_flagged_only = false;
        self.filters = [true; 8];
        self.pending_resolution = None;
    }
    pub fn is_complete(&self) -> bool {
        self.perk_lim == self.taken_count
    }
    pub fn update(&mut self, character: &mut Character) {
        self.taken_count = character.perks.iter().map(|p| p.ranks).sum();
        self.perk_lim = character.level + if character.has_trait(10) { 1 } else { 0 };
    }
    pub fn perk_filter_indices(perk: &PerkRow) -> Vec<usize> {
        let stat_reqs: Vec<&str> = perk.reqs
            .iter()
            .filter(|r| r.contains(':'))
            .map(|r| r
                .splitn(2,':')
                .next()
                .unwrap_or("")
                .trim())
            .collect();
        if stat_reqs.is_empty() {
            return vec![0];
        }
        stat_reqs.iter().map(|s| match s.to_lowercase().as_str() {
            "strength"     => 1,
            "perception"   => 2,
            "endurance"    => 3,
            "charisma"     => 4,
            "intelligence" => 5,
            "agility"      => 6,
            "luck"         => 7,
            _              => 0,
        }).collect()
    }
    pub fn perk_passes_filter(&self, perk: &PerkRow) -> bool {
        Self::perk_filter_indices(perk)
            .iter()
            .any(|&i| self.filters[i])
    }
    pub fn is_taken(&self, perk: &PerkRow, character: &Character,) -> bool {
        character.has_perk(perk.id)
    }
    pub fn is_flagged(&self, id: i32, character: &Character,) -> bool {
        character.flagged_perks.contains(&id)
    }
    pub fn is_eligible(&self, perk: &PerkRow, character: &Character,) -> bool {
        let taken = self.is_taken(perk, character);
        let taken_ranks = if taken { character.perks.iter().find(|p| p.id == perk.id).map(|p| p.ranks).unwrap_or(0).into() } else { 0 };
        //at max
        if taken_ranks >= perk.ranks { return false; }
        let next_rank_lvl = perk.level_req + taken_ranks * perk.rank_range;
        //doesn't meet level requirement for next rank
        if character.level < next_rank_lvl { return false }
        //special requirements
        for req in &perk.reqs {
            if req.contains(':') {
                let parts: Vec<&str> = req.splitn(2,':').collect();
                if parts.len() != 2 { continue; }
                let stat: &str = parts[0].trim();
                let val: i32 = parts[1].trim().parse().unwrap_or(0);
                let special = character.special.special_block();
                let meets = match stat {
                    "strength"     => { special[0].value >= val }
                    "perception"   => { special[1].value >= val }
                    "endurance"    => { special[2].value >= val }
                    "charisma"     => { special[3].value >= val }
                    "intelligence" => { special[4].value >= val }
                    "agility"      => { special[5].value >= val }
                    "luck"         => { special[6].value >= val }
                    _ => { true }
                };
                if !meets { return false }
            }
            //can't have read a book at character creation
            if req.trim().to_lowercase() == "book" {
                return false
            }
        }
        //other limits
        for limit in &perk.limits {
            let lower = limit.to_lowercase();
            if lower.contains("daring nature") && character.has_perk(25) ||
                lower.contains("cautious nature") && character.has_perk(18) ||
                lower.contains("robot") && character.is_robot() ||
                lower.contains("ghoul") && character.ghoul ||
                lower.contains("rads") && (character.is_robot() || character.ghoul || character.is_mutant()) ||
                lower.contains("companion") && character.companion != CompanionType::None { return false }
        }
        true
    }
    pub fn begin_resolve(&self, perk: &PerkRow, add: bool, name: String) -> Option<PerkResolutionPopup> {
        let resolution = match perk.id {
            12 => Some(PerkResolution::BwLk { version: None }),
            45 => Some(PerkResolution::IntenseTraining { selected_stat: None }),
            83 => Some(PerkResolution::Skilled { skill_a: None, skill_b: None }),
            92 => Some(PerkResolution::Tag { selected_skill: None }),
            110 => Some(PerkResolution::MmCf { version: None }),
            _ => None,
        };
        if name == "".to_string() {
            resolution.map(|r| PerkResolutionPopup { perk_id: perk.id, perk_name: perk.name.clone(), resolution: r, perk_add: add, open: true })
        } else {
            resolution.map(|r| PerkResolutionPopup { perk_id: perk.id, perk_name: name, resolution: r, perk_add: add, open: true })
        }
    }
    pub fn is_resolution_complete(popup: &PerkResolutionPopup) -> bool {
        match &popup.resolution {
            PerkResolution::BwLk { version } => version.is_some(),
            PerkResolution::IntenseTraining { selected_stat } => selected_stat.is_some(),
            PerkResolution::Skilled { skill_a, skill_b } => skill_a.is_some() && skill_b.is_some(),
            PerkResolution::Tag { selected_skill } => selected_skill.is_some(),
            PerkResolution::MmCf { version } => version.is_some(),
        }
    }
}

//background
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
    pub fn origin_backgrounds(&self, character: Character) -> Vec<(usize, &BackgroundRow)> {
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
    pub fn load_background(&mut self, db: &Db, index: usize) {
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
pub fn selections_complete(sels: &[SlotSelection]) -> bool {
    sels.iter().all(|s| match s {
        SlotSelection::Fixed => true,
        SlotSelection::Chosen(i) => *i != usize::MAX,
        SlotSelection::ManyForOneChosen(i) => *i != usize::MAX,
        SlotSelection::SingleOrDoubleChosen(take_single, double_picks) => *take_single || double_picks.iter().all(|p| p.is_some()),
        SlotSelection::SingleOrPackChosen(_) => true,
    })
}

//review
//for this i think we want to build the state to be something we can apply directly to the character struct upon acceptance; applying to the character directly here would likely lead to weird issues with clearing stuff when changing backgrounds/origins
pub struct ReviewState {
    pub loaded: bool,
    pub debug_load: bool,
}
impl ReviewState {
    pub fn new() -> Self {
        Self {
            loaded: false,
            debug_load: true,
        }
    }
}

//character sheet
#[derive(Clone, PartialEq)]
pub enum InventoryTab {
    Ammo,
    Apparel,
    Consumables,
    RobotModules,
    Gear,
    Misc,
}
pub struct InventoryState {
    pub open: bool,
    pub tab: InventoryTab,
    pub all_apparel: Vec<Apparel>,
    pub all_ammo: Vec<AmmoData>,
    pub all_consumables: Vec<Consumable>,
    pub all_modules: Vec<RobotModule>,
    pub all_gear: Vec<Gear>,
    pub filter: String,
    pub apparel_type_filter: Option<ApparelType>,
    pub misc_buf: String,
    pub ammo_qty: i32,
}
impl InventoryState {
    pub fn new() -> Self {
        Self {
            open: false,
            tab: InventoryTab::Apparel,
            all_apparel: vec![],
            all_ammo: vec![],
            all_consumables: vec![],
            all_modules: vec![],
            all_gear: vec![],
            filter: String::new(),
            apparel_type_filter: None,
            misc_buf: String::new(),
            ammo_qty: 1,
        }
    }
}
pub struct SheetState {
    pub origin_expanded: bool,
    pub background_expanded: bool,
    pub traits_expanded: bool,
    pub perks_expanded: Vec<bool>,
    pub notes_open: bool,
    pub notes_buf: String,
    pub xp_open: bool,
    pub xp_amount: i32,
    pub level: bool,
    pub up: bool,
    pub perks: Vec<PerkRow>,
    pub skill_choice: i32,
    pub perk_choice: i32,
    pub weapons_open: bool,
    pub weapon_list: Vec<(i32, String, String, String)>,
    pub weapon_filter: String,
    pub weapon_selected: Option<i32>,
    pub inventory: InventoryState,
}
impl SheetState {
    pub fn new() -> Self {
        Self {
            origin_expanded: false,
            background_expanded: false,
            traits_expanded: false,
            perks_expanded: vec![],
            notes_open: false,
            notes_buf: String::new(),
            xp_open: false,
            xp_amount: 0,
            level: false,
            up: true,
            perks: vec![],
            skill_choice: i32::MAX,
            perk_choice: i32::MAX,
            weapons_open: false,
            weapon_list: vec![],
            weapon_filter: String::new(),
            weapon_selected: None,
            inventory: InventoryState::new(),
        }
    }
    pub fn new_character(&mut self, character: &Character) {
        self.perks_expanded = character.perks.iter().map(|_| false).collect();
    }
}