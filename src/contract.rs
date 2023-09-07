#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw4::MemberResponse;
use cw4_group::msg::QueryMsg as Cw4QueryMsg;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::error::ContractError;
use crate::msg::{
    EntriesResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TallyVotesResponse, VotesResponse,
};
use crate::state::{
    Config, Entry, Votes, CATEGORIES, CATEGORY_ENTRIES, CONFIG, ENTRY_ID, ENTRY_VOTES,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:voting-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admins_cw4_group = deps.api.addr_validate(&msg.admins_cw4_group)?;
    let makers_cw4_group = deps.api.addr_validate(&msg.makers_cw4_group)?;

    let config = Config {
        admins_cw4_group,
        makers_cw4_group,
    };
    CONFIG.save(deps.storage, &config)?;

    CATEGORIES.save(deps.storage, &vec![])?;

    ENTRY_ID.save(deps.storage, &0)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddCategory { category } => execute_add_category(deps, env, info, category),
        ExecuteMsg::AddEntry {
            name,
            category,
            maker_addr,
            maker_name,
            breeder,
            genetics,
            farmer,
        } => execute_add_entry(
            deps, env, info, name, category, maker_addr, maker_name, breeder, genetics, farmer,
        ),
        ExecuteMsg::Vote {
            category,
            entry_id,
            votes,
        } => execute_vote(deps, env, info, category, entry_id, votes),
    }
}

fn execute_add_category(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    category: String,
) -> Result<Response, ContractError> {
    // Check if the sender is in admin cw4 group
    check_admin_membership(&deps, &info.sender)?;

    let mut categories = CATEGORIES.load(deps.storage)?;

    // Check if the category already exists
    if categories.contains(&category) {
        return Err(ContractError::InvalidCategory {});
    }
    categories.push(category.clone());

    CATEGORIES.save(deps.storage, &categories)?;

    Ok(Response::new().add_attribute("action", "add_category"))
}

fn execute_add_entry(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    category: String,
    maker_addr: String,
    maker_name: String,
    breeder: String,
    genetics: String,
    farmer: String,
) -> Result<Response, ContractError> {
    // Check if the sender is in admin cw4 group
    check_admin_membership(&deps, &info.sender)?;

    let categories = CATEGORIES.load(deps.storage)?;
    if !categories.contains(&category) {
        return Err(ContractError::InvalidCategory {});
    };

    let entry_id = (ENTRY_ID.load(deps.storage)?) + 1;

    let maker_addr = deps.api.addr_validate(&maker_addr)?;

    let entry = Entry {
        name,
        category: category.clone(),
        maker_addr,
        maker_name,
        breeder,
        genetics,
        farmer,
    };

    CATEGORY_ENTRIES.save(deps.storage, (category, entry_id), &entry)?;
    ENTRY_ID.save(deps.storage, &entry_id)?;

    Ok(Response::new().add_attribute("action", "add_entry"))
}

fn execute_vote(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    category: String,
    entry_id: u8,
    votes: Votes,
) -> Result<Response, ContractError> {
    // Check if the sender is in makers cw4 group
    check_maker_membership(&deps, &info.sender)?;

    let categories = CATEGORIES.load(deps.storage)?;
    // Check if the category is valid
    if !categories.contains(&category) {
        return Err(ContractError::InvalidCategory {});
    };

    let entry = CATEGORY_ENTRIES.load(deps.storage, (category.clone(), entry_id))?;

    // Check if the sender is not the same as the entry maker
    if info.sender == entry.maker_addr {
        return Err(ContractError::InvalidMaker {});
    };

    ENTRY_VOTES.save(deps.storage, (entry_id, info.sender), &votes)?;

    Ok(Response::new().add_attribute("action", "vote"))
}

fn check_admin_membership(deps: &DepsMut, sender: &Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check for admin cw4 group membership
    let res: MemberResponse = deps.querier.query_wasm_smart(
        config.admins_cw4_group,
        &Cw4QueryMsg::Member {
            addr: sender.to_string(),
            at_height: None,
        },
    )?;
    if res.weight.is_none() {
        return Err(ContractError::Unauthorized {});
    };

    Ok(())
}

fn check_maker_membership(deps: &DepsMut, sender: &Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check for admin cw4 group membership
    let res: MemberResponse = deps.querier.query_wasm_smart(
        config.makers_cw4_group,
        &Cw4QueryMsg::Member {
            addr: sender.to_string(),
            at_height: None,
        },
    )?;
    if res.weight.is_none() {
        return Err(ContractError::Unauthorized {});
    };

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Categories {} => to_binary(&query_categories(deps)?),
        QueryMsg::Entry { category, entry_id } => {
            to_binary(&query_entry(deps, category, entry_id)?)
        }
        QueryMsg::Entries {
            category,
            start_after,
            limit,
        } => to_binary(&query_entries(deps, category, start_after, limit)?),
        QueryMsg::TallyVotes {
            entry_id,
            start_after,
            limit,
        } => to_binary(&query_tally_votes(deps, entry_id, start_after, limit)?),
        QueryMsg::Votes {
            entry_id,
            maker_addr,
        } => to_binary(&query_votes(deps, entry_id, maker_addr)?),
    }
}

fn query_categories(deps: Deps) -> StdResult<Vec<String>> {
    let categories = CATEGORIES.load(deps.storage)?;
    Ok(categories)
}

fn query_entry(deps: Deps, category: String, entry_id: u8) -> StdResult<Entry> {
    let entry = CATEGORY_ENTRIES.load(deps.storage, (category, entry_id))?;
    Ok(entry)
}

fn query_entries(
    deps: Deps,
    category: String,
    start_after: Option<u8>,
    limit: Option<u8>,
) -> StdResult<Vec<EntriesResponse>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);

    let entries = CATEGORY_ENTRIES
        .prefix(category)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (entry_id, entry) = item.unwrap();
            EntriesResponse {
                id: entry_id,
                data: entry,
            }
        })
        .collect::<Vec<EntriesResponse>>();

    Ok(entries)
}

fn query_votes(deps: Deps, entry_id: u8, maker_addr: String) -> StdResult<Votes> {
    let addr = deps.api.addr_validate(&maker_addr)?;
    let votes = ENTRY_VOTES.load(deps.storage, (entry_id, addr))?;
    Ok(votes)
}

fn query_tally_votes(
    deps: Deps,
    entry_id: u8,
    start_after: Option<String>,
    limit: Option<u8>,
) -> StdResult<TallyVotesResponse> {
    let limit = limit.unwrap_or(30) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(Bound::exclusive);

    let mut sum = Votes {
        look: Uint128::zero(),
        smell: Uint128::zero(),
        taste: Uint128::zero(),
        post_melt: Uint128::zero(),
    };

    let votes = ENTRY_VOTES
        .prefix(entry_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (maker_addr, votes) = item.unwrap();
            sum.look += votes.look;
            sum.smell += votes.smell;
            sum.taste += votes.taste;
            sum.post_melt += votes.post_melt;
            VotesResponse {
                entry_id,
                maker_addr: maker_addr.to_string(),
                votes: votes.clone(),
                sum: votes.look + votes.smell + votes.taste + votes.post_melt,
            }
        })
        .collect::<Vec<VotesResponse>>();

    let response = TallyVotesResponse { votes, sum };

    Ok(response)
}

#[cfg(test)]
mod tests {}
