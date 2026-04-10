use uuid::Uuid;

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
        self.traits.iter().any(|t| {
            t.id == 7
        })
    }
    pub fn is_mutant(&self) -> bool {
        self.mutant != MutantType::None
    }
    pub fn is_robot(&self) -> bool {
        self.robot != RobotType::None
    }
}

pub struct Player {
    id: Uuid,
    name: String,
}

impl Player {
    pub fn new() -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
        }
    }
}

pub struct Party {
    id: Uuid,
    name: String,
    ap_players: i32,
    ap_gm: i32,
}

impl Party {
    pub fn new() -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
            ap_players: 0,
            ap_gm: 0,
        }
    }
}

pub struct Origin {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub can_ghoul: bool,
}

pub struct Background {
    id: i32,
    name: String,
    desc: String,
}

pub struct Trait {
    pub id: i32,
    pub name: String,
    pub desc: String,
}

#[derive(PartialEq)]
pub enum MutantType {
    None,
    SuperMutant,
    Nightkin,
}

#[derive(PartialEq)]
pub enum RobotType {
    None,
    Handy,
    Protectron,
    Robobrain,
    Securitron,
    Synth,
    Assaultron,
}

pub enum SpecialAttr {
    Strength,
    Perception,
    Endurance,
    Charisma,
    Intelligence,
    Agility,
    Luck,
}

pub struct Special {
    strength: SpecialBlock,
    perception: SpecialBlock,
    endurance: SpecialBlock,
    charisma: SpecialBlock,
    intelligence: SpecialBlock,
    agility: SpecialBlock,
    luck: SpecialBlock,
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
}

pub struct SpecialBlock {
    value: i32,
    gifted: bool,
    trained: i32,
    max: i32,
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
}

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

pub struct Skills {
    athletics: SkillBlock,
	barter: SkillBlock,
	big_guns: SkillBlock,
	energy_weapons: SkillBlock,
	explosives: SkillBlock,
    lockpick: SkillBlock,
	medicine: SkillBlock,
	melee_weapons: SkillBlock,
	pilot: SkillBlock,
	repair: SkillBlock,
    science: SkillBlock,
	small_guns: SkillBlock,
	sneak: SkillBlock,
	speech: SkillBlock,
	survival: SkillBlock,
    throwing: SkillBlock,
	unarmed: SkillBlock,
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
}

pub struct SkillBlock {
    ranks: i32,
    tagged: TagType,
    skilled: Vec<i32>,
    total: i32,
    max: i32,
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
}

pub enum TagType {
    None,
    Trait,
    Perk,
    Standard,
}

pub struct Perk {
    id: i32,
    name: String,
    desc: Vec<String>,
    ranks: i32,
}

pub struct MeleeModifiers {
    melee: i32,
    unarmed: i32,
    sneak: i32,
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

pub struct Limbs {
    head: Limb,
    torso: Limb,
    body: Limb,
    arm_left: Limb,
    arm_right: Limb,
    leg_left: Limb,
    leg_right: Limb,
    optics: Limb,
    arm_1: Limb,
    arm_2: Limb,
    arm_3: Limb,
    thruster: Limb,
    wheel: Limb,
    track_left: Limb,
    track_right: Limb,
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

pub struct Limb {
    active: bool,
    ph_dr: i32,
    en_dr: i32,
    rd_dr: i32,
    injuries: i32,
    equipped: Option<Apparel>,
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

pub struct Weapon {
    id: i32,
    name: String,
    prefix: String,
    skill: Skill,
    target: i32,
    tag: bool,
    damage: i32,
    effects: Vec<String>, // new struct?
    dam_type: DamageType,
    rate: i32,
    range: String,
    qualities: Vec<String>, // new struct?
    ammo: String,
    wgt: i32,
    mods: Vec<WeaponMods>,
}

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

pub struct WeaponMods {
    slot: WeaponSlot,
    installed: bool,
    id: i32,
    name: String,
    prefix: String,
    wgt: i32,
    damage_set: i32,
    damage_chg: i32,
    rate_set: i32,
    rate_chg: i32,
    range_set: i32,
    range_chg: i32,
    ammo_set: AmmoData,
    effect_add: Vec<String>,
    effect_rem: Vec<String>,
    quality_add: Vec<String>,
    quality_rem: Vec<String>,
    slot_add: WeaponSlot,
    damage_type_set: DamageType,
    weapon_add: Weapon,
    special_ability: String,
}

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

pub struct AmmoData {
    id: i32,
    name: String,
    wgt: i32
}

pub struct Ammo {
    ammo: AmmoData,
    variants: Vec<AmmoData>,
}

pub struct AmmoInv {
    ammo: AmmoData,
    quantity: i32,
}

pub struct Apparel {
    id: i32,
    name: String,
    prefix: String,
    apparel_type: ApparelType,
    ph_dr: i32,
    en_dr: i32,
    rd_dr: i32,
    wgt: i32,
    effects: Vec<String>,
    covers: Vec<BodyLocation>,
    equipped: bool,
}

pub enum ApparelType {
    Clothing,
    Outfit,
    Headgear,
    Armor,
    PowerArmor,
    RobotArmor,
}

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

pub struct RobotModule {
    id: i32,
    name: String,
    installed: bool,
    effect: Vec<String>,
    wgt: i32,
}

pub struct Consumable {
    id: i32,
    name: String,
    consumable_type: ConsumableType,
    health: i32,
    effects: Vec<String>,
    rads: i32,
    wgt: i32,
    duration: String,
    addiction: i32,
    quantity: i32,
}

pub enum ConsumableType {
    Chem,
    Food,
    Beverage,
    Other,
    Publication,
}

pub struct Gear {
    id: i32,
    name: String,
    effect: Vec<String>,
    wgt: i32,
    quantity: i32,
}

pub struct Junk {
    common: i32,
    uncommon: i32,
    rare: i32,
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