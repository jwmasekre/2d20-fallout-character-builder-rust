use crate::structs::{
    AppScreen,
    Version,
    PreRelease,
};

pub const CONFIG_FILE: &str = "usr_config.toml";
pub const VERSION: Version = Version {
    major: 0,
    minor: 6,
    patch: 2,
    prerelease: PreRelease::Alpha,
    prerelease_ver: 0,
};
pub const DATE: &str = "20260528";
pub const BUILD_SCREENS: &[(AppScreen, &str)] = &[
    (AppScreen::OriginSelect, "Origin"),
    (AppScreen::SpecialAssignment, "SPECIAL"),
    (AppScreen::SkillAssignment, "Skills"),
    (AppScreen::PerkSelect, "Perks"),
    (AppScreen::StatCalculation, "Stats"),
    (AppScreen::BackgroundSelect, "Background"),
    (AppScreen::CharacterReview, "Review"),
];
pub const NULL_PARTY: &str = "00000000-0000-0000-0000-000000000000";
pub const SPECIAL_LABELS: [&str; 7] = ["Strength", "Perception", "Endurance", "Charisma", "Intelligence", "Agility", "Luck"];
pub const SKILLS: [&str; 17] = [
    "Athletics", "Barter", "Big Guns", "Energy Weapons", "Explosives",
    "Lockpick", "Medicine", "Melee Weapons", "Pilot", "Repair",
    "Science", "Small Guns", "Sneak", "Speech", "Survival",
    "Throwing", "Unarmed",
];