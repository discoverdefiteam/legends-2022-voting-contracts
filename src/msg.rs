use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::{Entry, Votes};

#[cw_serde]
pub struct InstantiateMsg {
    pub admins_cw4_group: String,
    pub makers_cw4_group: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddCategory {
        category: String,
    },
    AddEntry {
        name: String,
        category: String,
        maker_addr: String,
        maker_name: String,
        breeder: String,
        genetics: String,
        farmer: String,
    },
    Vote {
        category: String,
        entry_id: u8,
        votes: Votes,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<String>)]
    Categories {},
    #[returns(Entry)]
    Entry { category: String, entry_id: u8 },
    #[returns(Vec<EntriesResponse>)]
    Entries {
        category: String,
        start_after: Option<u8>,
        limit: Option<u8>,
    },
    #[returns(Votes)]
    Votes { entry_id: u8, maker_addr: String },
    #[returns(Vec<TallyVotesResponse>)]
    TallyVotes {
        entry_id: u8,
        start_after: Option<String>,
        limit: Option<u8>,
    },
}

#[cw_serde]
pub struct EntriesResponse {
    pub id: u8,
    pub data: Entry,
}

#[cw_serde]
pub struct VotesResponse {
    pub entry_id: u8,
    pub maker_addr: String,
    pub votes: Votes,
    pub sum: Uint128,
}

#[cw_serde]
pub struct TallyVotesResponse {
    pub votes: Vec<VotesResponse>,
    pub sum: Votes,
}
