use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Background {
    pub id: u16,
    pub name: String,
    pub origin_id: u16,
    pub caps: u16,
    pub misc: String,
    pub trinket: u16,
    pub food: u16,
    pub forage: u16,
    pub bev: u16,
    pub chem: u16,
    pub ammo: u16,
    pub aid: u16,
    pub odd: u8,
    pub outcast: u8,
    pub junk: u8,
    pub sourcebook_id: u8,
}