use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admins_cw4_group: Addr,
    pub makers_cw4_group: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Votes {
    pub look: Uint128,
    pub smell: Uint128,
    pub taste: Uint128,
    pub post_melt: Uint128,
}

#[cw_serde]
pub struct Entry {
    pub name: String,
    pub category: String,
    pub maker_addr: Addr,
    pub maker_name: String,
    pub breeder: String,
    pub genetics: String,
    pub farmer: String,
}
pub const ENTRY_ID: Item<u8> = Item::new("entry_id");

// Vector of category names
pub const CATEGORIES: Item<Vec<String>> = Item::new("categories");

// (Category Names, Entry IDs) -> Entry
pub const CATEGORY_ENTRIES: Map<(String, u8), Entry> = Map::new("category_entries");

// (Entry IDs, Maker Addr) -> Votes
pub const ENTRY_VOTES: Map<(u8, Addr), Votes> = Map::new("entry_votes");
