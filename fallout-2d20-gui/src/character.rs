use uuid::Uuid;

use crate::screens::special_assignment::SpecialState;

#[derive(Clone)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub player: Player,
    pub party: Option<Party>,
    pub level: i32,
    pub xp: i32,
    pub origin: Option<Origin>,
    pub background: Option<Background>,
    pub traits: Vec<Trait>,
    pub ghoul: bool,
    pub mutant: MutantType,
    pub robot: RobotType,
    pub companion: CompanionType,
    pub robot_hat: Option<Apparel>,
    pub special: Special,
    pub luck_points: i32,
    pub luck_points_max: i32,
    pub rad_points: i32,
    pub skills: Skills,
    pub perks: Vec<Perk>,
    pub melee_mod: MeleeModifiers,
    pub defense: i32,
    pub initiative: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub poison_dr: i32,
    pub limb_dr: Limbs,
    pub weapons: Vec<Weapon>,
    pub ammo: Vec<AmmoInv>,
    pub apparel: Vec<Apparel>,
    pub robot_modules: Vec<RobotModule>,
    pub consumables: Vec<Consumable>,
    pub gear: Vec<Gear>,
    pub junk: Junk,
    pub misc: Vec<String>,
    pub carry_wgt: i32,
    pub carry_wgt_max: i32,
    pub notes: String,
}

impl Character {
    pub fn new(player: Player, party: Option<Party>) -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
            player: player,
            party: party,
            level: 1,
            xp: 0,
            origin: None,
            background: None,
            traits: vec![],
            ghoul: false,
            mutant: MutantType::None,
            robot: RobotType::None,
            companion: CompanionType::None,
            robot_hat: None,
            special: Special::new(),
            luck_points: 5,
            luck_points_max: 5,
            rad_points: 0,
            skills: Skills::new(),
            perks: vec![],
            melee_mod: MeleeModifiers::new(),
            defense: 0,
            initiative: 10,
            hp: 10,
            hp_max: 10,
            poison_dr: 0,
            limb_dr: Limbs::new(),
            weapons: vec![],
            ammo: vec![],
            apparel: vec![],
            robot_modules: vec![],
            consumables: vec![],
            gear: vec![],
            junk: Junk::new(),
            misc: vec![],
            carry_wgt: 0,
            carry_wgt_max: 200,
            notes: String::new(),
        }
    }
    pub fn is_gifted(&self) -> bool {
        self.has_trait(7)
    }
    pub fn is_mutant(&self) -> bool {
        self.mutant != MutantType::None
    }
    pub fn is_robot(&self) -> bool {
        self.robot != RobotType::None
    }
    pub fn total_skill(&self) -> i32 {
        self.skills.skill_block().iter().map(|s| s.total).sum()
    }
    pub fn total_skill_ranks(&self) -> i32 {
        self.skills.skill_block().iter().map(|s| s.ranks).sum()
    }
    pub fn has_trait(&self, id: i32) -> bool {
        self.traits.iter().any(|t| t.id == id)
    }
    pub fn has_any_trait(&self, id: Vec<i32>) -> bool {
        self.traits.iter().any(|t| id.contains(&t.id))
    }
    pub fn has_perk(&self, id: i32) -> bool {
        self.perks.iter().any(|p| p.id == id)
    }
    pub fn perk_ranks(&self, id: i32) -> i32 {
        self.perks.iter().find(|p| p.id == id).map(|p| p.ranks).unwrap_or(0)
    }

}

#[derive(Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
}

impl Player {
    pub fn new() -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct Party {
    pub id: Uuid,
    pub name: String,
    pub ap_players: i32,
    pub ap_gm: i32,
    pub max_ap: i32,
}

impl Party {
    pub fn new() -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
            ap_players: 0,
            ap_gm: 0,
            max_ap: 6,
        }
    }
}

#[derive(Clone)]
pub struct Origin {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub can_ghoul: bool,
}

#[derive(Clone)]
pub struct Background {
    pub id: i32,
    pub name: String,
    pub desc: String,
}

#[derive(Clone)]
pub struct Trait {
    pub id: i32,
    pub name: String,
    pub desc: String,
}

#[derive(PartialEq, Clone)]
pub enum MutantType {
    None,
    SuperMutant,
    Nightkin,
}

#[derive(PartialEq, Clone)]
pub enum RobotType {
    None,
    Handy,
    Protectron,
    Robobrain,
    Securitron,
    Synth,
    Assaultron,
}

#[derive(PartialEq, Clone)]
pub enum CompanionType {
    None,
    Dogmeat,
    Human,
    Robot,
}

#[derive(Clone)]
pub enum SpecialAttr {
    Strength,
    Perception,
    Endurance,
    Charisma,
    Intelligence,
    Agility,
    Luck,
}

#[derive(Clone)]
pub struct Special {
    pub strength: SpecialBlock,
    pub perception: SpecialBlock,
    pub endurance: SpecialBlock,
    pub charisma: SpecialBlock,
    pub intelligence: SpecialBlock,
    pub agility: SpecialBlock,
    pub luck: SpecialBlock,
}

impl Special {
    fn new() -> Self {
        Self {
            strength: SpecialBlock::new(),
            perception: SpecialBlock::new(),
            endurance: SpecialBlock::new(),
            charisma: SpecialBlock::new(),
            intelligence: SpecialBlock::new(),
            agility: SpecialBlock::new(),
            luck: SpecialBlock::new(),
        }
    }
    pub fn apply_max(character: &mut Character) {
        match character.mutant {
            MutantType::None => {
                character.special.intelligence.max = 10;
                character.special.charisma.max = 10;
                character.special.strength.max = 10;
                character.special.endurance.max = 10;
                return
            },
            MutantType::SuperMutant => {
                character.special.intelligence.max = 6;
                character.special.charisma.max = 6;
            },
            MutantType::Nightkin => {
                character.special.intelligence.max = 8;
                character.special.charisma.max = 8;
            }
        }
        character.special.strength.max = 12;
        character.special.endurance.max = 12;
    }
    pub fn mut_special_block(&mut self) -> [&mut SpecialBlock; 7] {
        [
            &mut self.strength,
            &mut self.perception,
            &mut self.endurance,
            &mut self.charisma,
            &mut self.intelligence,
            &mut self.agility,
            &mut self.luck,
        ]
    }
    pub fn special_block(&self) -> [SpecialBlock; 7] {
        [
            self.strength.clone(),
            self.perception.clone(),
            self.endurance.clone(),
            self.charisma.clone(),
            self.intelligence.clone(),
            self.agility.clone(),
            self.luck.clone(),
        ]
    }
}

#[derive(Clone)]
pub struct SpecialBlock {
    pub value: i32,
    pub gifted: bool,
    pub trained: i32,
    pub max: i32,
}

impl SpecialBlock {
    fn new() -> Self {
        Self {
            value: 5,
            gifted: false,
            trained: 0,
            max: 10,
        }
    }
    pub fn can_increase(&self, state: &SpecialState, character: &Character) -> bool {
        self.value < self.max && state.remaining_points(character) > 0
    }
    pub fn can_decrease(&self, character: &Character) -> bool {
        self.value > 4 + if character.is_mutant() { 2 } else { 0 }
    }
}

#[derive(Clone)]
pub enum Skill {
    Athletics,
	Barter,
	BigGuns,
	EnergyWeapons,
	Explosives,
    Lockpick,
	Medicine,
	MeleeWeapons,
	Pilot,
	Repair,
    Science,
	SmallGuns,
	Sneak,
	Speech,
	Survival,
    Throwing,
	Unarmed,
}

#[derive(Clone)]
pub struct Skills {
    pub athletics: SkillBlock,
	pub barter: SkillBlock,
	pub big_guns: SkillBlock,
	pub energy_weapons: SkillBlock,
	pub explosives: SkillBlock,
    pub lockpick: SkillBlock,
	pub medicine: SkillBlock,
	pub melee_weapons: SkillBlock,
	pub pilot: SkillBlock,
	pub repair: SkillBlock,
    pub science: SkillBlock,
	pub small_guns: SkillBlock,
	pub sneak: SkillBlock,
	pub speech: SkillBlock,
	pub survival: SkillBlock,
    pub throwing: SkillBlock,
	pub unarmed: SkillBlock,
}

impl Skills {
    fn new() -> Self {
        Self {
            athletics: SkillBlock::new(),
            barter: SkillBlock::new(),
            big_guns: SkillBlock::new(),
            energy_weapons: SkillBlock::new(),
            explosives: SkillBlock::new(),
            lockpick: SkillBlock::new(),
            medicine: SkillBlock::new(),
            melee_weapons: SkillBlock::new(),
            pilot: SkillBlock::new(),
            repair: SkillBlock::new(),
            science: SkillBlock::new(),
            small_guns: SkillBlock::new(),
            sneak: SkillBlock::new(),
            speech: SkillBlock::new(),
            survival: SkillBlock::new(),
            throwing: SkillBlock::new(),
            unarmed: SkillBlock::new(),
        }
    }
    pub fn skill_block(&self) -> [SkillBlock; 17] {
        [
            self.athletics.clone(),
            self.barter.clone(),
            self.big_guns.clone(),
            self.energy_weapons.clone(),
            self.explosives.clone(),
            self.lockpick.clone(),
            self.medicine.clone(),
            self.melee_weapons.clone(),
            self.pilot.clone(),
            self.repair.clone(),
            self.science.clone(),
            self.small_guns.clone(),
            self.sneak.clone(),
            self.speech.clone(),
            self.survival.clone(),
            self.throwing.clone(),
            self.unarmed.clone(),
        ]
    }
    pub fn mut_skill_block(&mut self) -> [&mut SkillBlock; 17] {
        [
            &mut self.athletics,
            &mut self.barter,
            &mut self.big_guns,
            &mut self.energy_weapons,
            &mut self.explosives,
            &mut self.lockpick,
            &mut self.medicine,
            &mut self.melee_weapons,
            &mut self.pilot,
            &mut self.repair,
            &mut self.science,
            &mut self.small_guns,
            &mut self.sneak,
            &mut self.speech,
            &mut self.survival,
            &mut self.throwing,
            &mut self.unarmed,
        ]
    }
    pub fn standard_tags(&self) -> i32 {
        self.skill_block().iter().filter(|t| t.tagged == TagType::Standard).count() as i32
    }
    pub fn trait_tags(&self) -> i32 {
        self.skill_block().iter().filter(|t| t.tagged == TagType::Trait).count() as i32
    }
    pub fn perk_tags(&self) -> i32 {
        self.skill_block().iter().filter(|t| t.tagged == TagType::Perk).count() as i32
    }
    pub fn total_tags(&self) -> i32 {
        self.standard_tags() + self.trait_tags() + self.perk_tags()
    }
    pub fn zip_skilled(&self) -> Vec<(usize,usize)> {
        let mut zipped = vec![];
        let skilled_count = self.athletics.skilled.len();
        for rank in 0..skilled_count {
            let mut sk_a: usize = 17;
            for (i, skill) in self.skill_block().iter().enumerate() {
                match skill.is_skilled(rank) {
                    1 => {if sk_a == 17 {sk_a = i} else {zipped.push((sk_a,i)); continue}},
                    2 => {zipped.push((i,i)); continue},
                    _ => {},
                }
            }
        }
        zipped
    }
    pub fn available_tags(&self, character: &Character) -> Vec<usize> {
        let mut available = vec![];
        for (i,skill) in self.skill_block().iter().enumerate() {
            if !skill.is_tagged() && !(i == 10 && character.has_trait(27)) { available.push(i) }
        }
        available
    }
    pub fn perk_tagged(&self) -> Vec<usize> {
        let mut tagged = vec![];
        for (i,skill) in self.skill_block().iter().enumerate() {
            if skill.tagged == TagType::Perk {tagged.push(i)}
        }
        tagged
    }
}

#[derive(Clone)]
pub struct SkillBlock {
    pub ranks: i32,
    pub tagged: TagType,
    //skilled will create a new entry in every skill, either 0, 1 or 2. this aligns the skill selections with each rank of the perk
    pub skilled: Vec<i32>,
    pub total: i32,
    pub max: i32,
}

impl SkillBlock {
    fn new() -> Self {
        Self {
            ranks: 0,
            tagged: TagType::None,
            skilled: vec![],
            total: 0,
            max: 3,
        }
    }
    pub fn is_tagged(&self) -> bool {
        self.tagged != TagType::None
    }
    pub fn update(&mut self) {
        self.total = self.ranks + if self.is_tagged() { 2 } else { 0 };
    }
    pub fn is_skilled(&self, rank: usize) -> i32 {
        self.skilled[rank]
    }
}

#[derive(PartialEq, Clone)]
pub enum TagType {
    None,
    Trait,
    Perk,
    Standard,
}

#[derive(Clone)]
pub struct Perk {
    pub id: i32,
    pub name: String,
    pub desc: Vec<String>,
    pub ranks: i32,
}

#[derive(Clone)]
pub struct MeleeModifiers {
    pub melee: i32,
    pub unarmed: i32,
    pub sneak: i32,
}

impl MeleeModifiers {
    fn new() -> Self {
        Self {
            melee: 0,
            unarmed: 0,
            sneak: 0,
        }
    }
}

#[derive(Clone)]
pub struct Limbs {
    pub head: Limb,
    pub torso: Limb,
    pub body: Limb,
    pub arm_left: Limb,
    pub arm_right: Limb,
    pub leg_left: Limb,
    pub leg_right: Limb,
    pub optics: Limb,
    pub arm_1: Limb,
    pub arm_2: Limb,
    pub arm_3: Limb,
    pub thruster: Limb,
    pub wheel: Limb,
    pub track_left: Limb,
    pub track_right: Limb,
}

impl Limbs {
    fn new() -> Self {
        Self {
            head: Limb::new_active(),
            torso: Limb::new_active(),
            body: Limb::new_inactive(),
            arm_left: Limb::new_active(),
            arm_right: Limb::new_active(),
            leg_left: Limb::new_active(),
            leg_right: Limb::new_active(),
            optics: Limb::new_inactive(),
            arm_1: Limb::new_inactive(),
            arm_2: Limb::new_inactive(),
            arm_3: Limb::new_inactive(),
            thruster: Limb::new_inactive(),
            wheel: Limb::new_inactive(),
            track_left: Limb::new_inactive(),
            track_right: Limb::new_inactive(),
        }
    }
}

#[derive(Clone)]
pub struct Limb {
    pub active: bool,
    pub ph_dr: i32,
    pub en_dr: i32,
    pub rd_dr: i32,
    pub injuries: i32,
    pub equipped: Option<Apparel>,
}

impl Limb {
    fn new_active() -> Self {
        Self {
            active: true,
            ph_dr: 0,
            en_dr: 0,
            rd_dr: 0,
            injuries: 0,
            equipped: None,
        }
    }
    fn new_inactive() -> Self {
        Self {
            active: false,
            ph_dr: 0,
            en_dr: 0,
            rd_dr: 0,
            injuries: 0,
            equipped: None,
        }
    }
}

#[derive(Clone)]
pub struct Weapon {
    pub id: i32,
    pub name: String,
    pub prefix: String,
    pub skill: Skill,
    pub target: i32,
    pub tag: bool,
    pub damage: i32,
    pub effects: Vec<String>, // new struct?
    pub dam_type: DamageType,
    pub rate: i32,
    pub range: String,
    pub qualities: Vec<String>, // new struct?
    pub ammo: String,
    pub wgt: i32,
    pub mods: Vec<WeaponMods>,
}

#[derive(Clone)]
pub enum DamageType {
    Ph,
    En,
    PhEn,
    Rad,
    EnRad,
    Poi,
    All,
    None,
}

#[derive(Clone)]
pub struct WeaponMods {
    pub slot: WeaponSlot,
    pub installed: bool,
    pub id: i32,
    pub name: String,
    pub prefix: String,
    pub wgt: i32,
    pub damage_set: i32,
    pub damage_chg: i32,
    pub rate_set: i32,
    pub rate_chg: i32,
    pub range_set: i32,
    pub range_chg: i32,
    pub ammo_set: AmmoData,
    pub effect_add: Vec<String>,
    pub effect_rem: Vec<String>,
    pub quality_add: Vec<String>,
    pub quality_rem: Vec<String>,
    pub slot_add: WeaponSlot,
    pub damage_type_set: DamageType,
    pub weapon_add: Weapon,
    pub special_ability: String,
}

#[derive(Clone)]
pub enum WeaponSlot {
    None,
    Receiver,
    Barrel,
    Stock,
    Grip,
    Magazine,
    Sights,
    Muzzle,
    Capacitors,
    Dish,
    Fuel,
    Tank,
    Nozzle,
    Blade,
    Blunt,
    Frame,
}

#[derive(Clone)]
pub struct AmmoData {
    pub id: i32,
    pub name: String,
    pub wgt: i32
}

#[derive(Clone)]
pub struct Ammo {
    pub ammo: AmmoData,
    pub variants: Vec<AmmoData>,
}

#[derive(Clone)]
pub struct AmmoInv {
    pub ammo: AmmoData,
    pub quantity: i32,
}

#[derive(Clone)]
pub struct Apparel {
    pub id: i32,
    pub name: String,
    pub prefix: String,
    pub apparel_type: ApparelType,
    pub ph_dr: i32,
    pub en_dr: i32,
    pub rd_dr: i32,
    pub wgt: i32,
    pub effects: Vec<String>,
    pub covers: Vec<BodyLocation>,
    pub equipped: bool,
}

#[derive(Clone)]
pub enum ApparelType {
    Clothing,
    Outfit,
    Headgear,
    Armor,
    PowerArmor,
    RobotArmor,
}

#[derive(Clone)]
pub enum BodyLocation {
    None,
    Head,
    ArmLeft,
    ArmRight,
    Torso,
    LegLeft,
    LegRight,
    Optics,
    Arm1,
    Arm2,
    Arm3,
    Body,
    Thruster,
    Wheel,
}

#[derive(Clone)]
pub struct RobotModule {
    pub id: i32,
    pub name: String,
    pub installed: bool,
    pub effect: Vec<String>,
    pub wgt: i32,
}

#[derive(Clone)]
pub struct Consumable {
    pub id: i32,
    pub name: String,
    pub consumable_type: ConsumableType,
    pub health: i32,
    pub effects: Vec<String>,
    pub rads: i32,
    pub wgt: i32,
    pub duration: String,
    pub addiction: i32,
    pub quantity: i32,
}

#[derive(Clone)]
pub enum ConsumableType {
    Chem,
    Food,
    Beverage,
    Other,
    Publication,
}

#[derive(Clone)]
pub struct Gear {
    pub id: i32,
    pub name: String,
    pub effect: Vec<String>,
    pub wgt: i32,
    pub quantity: i32,
}

#[derive(Clone)]
pub struct Junk {
    pub common: i32,
    pub uncommon: i32,
    pub rare: i32,
}

impl Junk {
    fn new() -> Self {
        Self {
            common: 0,
            uncommon: 0,
            rare: 0,
        }
    }
}