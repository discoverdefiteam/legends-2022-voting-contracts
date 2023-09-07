use crate::{
    msg::{EntriesResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Entry, Votes},
    ContractError,
};
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw4::Member;
use cw4_group::msg::InstantiateMsg as Cw4InstantiateMsg;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

pub fn voting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn cw4_group_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno..user";
const ADMIN: &str = "juno..admin";
const FIRST_MAKER: &str = "juno..firstmaker";
const SECOND_MAKER: &str = "juno..secondmaker";

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(ADMIN),
                vec![Coin {
                    denom: "denom".to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
    })
}

fn setup_contract(app: &mut App, admins_cw4_group: String, makers_cw4_group: String) -> Addr {
    let code_id = app.store_code(voting_contract());
    app.instantiate_contract(
        code_id,
        Addr::unchecked(ADMIN),
        &InstantiateMsg {
            admins_cw4_group,
            makers_cw4_group,
        },
        &vec![],
        "Voting Contract",
        None,
    )
    .unwrap()
}

fn setup_cw4_group(app: &mut App, members: Vec<&str>) -> Addr {
    let code_id = app.store_code(cw4_group_contract());
    let members = members
        .iter()
        .map(|m| Member {
            addr: m.to_string(),
            weight: 1,
        })
        .collect();
    app.instantiate_contract(
        code_id,
        Addr::unchecked(ADMIN),
        &Cw4InstantiateMsg {
            admin: Some(ADMIN.to_string()),
            members,
        },
        &vec![],
        "CW4 Group",
        None,
    )
    .unwrap()
}

fn setup_category(app: &mut App, contract_addr: Addr, category: String) {
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr,
        &ExecuteMsg::AddCategory { category },
        &vec![],
    )
    .unwrap();
}

fn setup_entry(
    app: &mut App,
    contract_addr: Addr,
    name: String,
    category: String,
    maker_addr: String,
    maker_name: String,
    breeder: String,
    genetics: String,
    farmer: String,
) {
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr,
        &ExecuteMsg::AddEntry {
            name,
            category,
            maker_addr,
            maker_name,
            breeder,
            genetics,
            farmer,
        },
        &vec![],
    )
    .unwrap();
}

fn setup_vote(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    category: String,
    entry_id: u8,
    votes: Votes,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::Vote {
            category,
            entry_id,
            votes,
        },
        &vec![],
    )
    .unwrap();
}

mod execute {
    use super::*;

    mod add_category {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                "contract_address".to_string(),
            );

            app.execute_contract(
                Addr::unchecked(ADMIN),
                contract_addr.clone(),
                &ExecuteMsg::AddCategory {
                    category: String::from("category_1"),
                },
                &vec![],
            )
            .unwrap();

            let res: Vec<String> = app
                .wrap()
                .query_wasm_smart(contract_addr, &QueryMsg::Categories {})
                .unwrap();
            assert_eq!(res, vec![String::from("category_1")]);
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                "contract_address".to_string(),
            );

            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    contract_addr.clone(),
                    &ExecuteMsg::AddCategory {
                        category: String::from("category_1"),
                    },
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_invalid_category() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                "contract_address".to_string(),
            );

            app.execute_contract(
                Addr::unchecked(ADMIN),
                contract_addr.clone(),
                &ExecuteMsg::AddCategory {
                    category: String::from("category_1"),
                },
                &vec![],
            )
            .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    contract_addr.clone(),
                    &ExecuteMsg::AddCategory {
                        category: String::from("category_1"),
                    },
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidCategory {}.to_string()
            );
        }
    }

    mod add_entry {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                "contract_address".to_string(),
            );

            setup_category(&mut app, contract_addr.clone(), "category_1".to_string());

            app.execute_contract(
                Addr::unchecked(ADMIN),
                contract_addr.clone(),
                &ExecuteMsg::AddEntry {
                    name: "entry_name".to_string(),
                    category: "category_1".to_string(),
                    maker_addr: FIRST_MAKER.to_string(),
                    maker_name: "maker_name".to_string(),
                    breeder: "entry_breeder".to_string(),
                    genetics: "entry_genetics".to_string(),
                    farmer: "entry_farmer".to_string(),
                },
                &vec![],
            )
            .unwrap();

            let res: Vec<EntriesResponse> = app
                .wrap()
                .query_wasm_smart(
                    contract_addr,
                    &QueryMsg::Entries {
                        category: "category_1".to_string(),
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();
            assert_eq!(
                res,
                vec![EntriesResponse {
                    id: 1,
                    data: Entry {
                        name: "entry_name".to_string(),
                        category: "category_1".to_string(),
                        maker_addr: Addr::unchecked(FIRST_MAKER.to_string()),
                        maker_name: "maker_name".to_string(),
                        breeder: "entry_breeder".to_string(),
                        genetics: "entry_genetics".to_string(),
                        farmer: "entry_farmer".to_string(),
                    }
                }]
            );
        }

        #[test]
        fn test_invalid_category() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                "contract_address".to_string(),
            );

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    contract_addr.clone(),
                    &ExecuteMsg::AddEntry {
                        name: "entry_name".to_string(),
                        category: "category_1".to_string(),
                        maker_addr: FIRST_MAKER.to_string(),
                        maker_name: "maker_name".to_string(),
                        breeder: "entry_breeder".to_string(),
                        genetics: "entry_genetics".to_string(),
                        farmer: "entry_farmer".to_string(),
                    },
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidCategory {}.to_string()
            );
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                "contract_address".to_string(),
            );

            setup_category(&mut app, contract_addr.clone(), "category_1".to_string());

            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    contract_addr.clone(),
                    &ExecuteMsg::AddEntry {
                        name: "entry_name".to_string(),
                        category: "category_1".to_string(),
                        maker_addr: FIRST_MAKER.to_string(),
                        maker_name: "maker_name".to_string(),
                        breeder: "entry_breeder".to_string(),
                        genetics: "entry_genetics".to_string(),
                        farmer: "entry_farmer".to_string(),
                    },
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod vote {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let makers_cw4_group = setup_cw4_group(&mut app, vec![FIRST_MAKER, SECOND_MAKER]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                makers_cw4_group.to_string(),
            );

            setup_category(&mut app, contract_addr.clone(), "category_1".to_string());
            setup_entry(
                &mut app,
                contract_addr.clone(),
                "entry_name".to_string(),
                "category_1".to_string(),
                FIRST_MAKER.to_string(),
                "maker_name".to_string(),
                "entry_breeder".to_string(),
                "entry_genetics".to_string(),
                "entry_farmer".to_string(),
            );

            let votes = Votes {
                look: Uint128::new(775),
                smell: Uint128::new(820),
                taste: Uint128::new(1000),
                post_melt: Uint128::new(250),
            };

            app.execute_contract(
                Addr::unchecked(SECOND_MAKER),
                contract_addr.clone(),
                &ExecuteMsg::Vote {
                    category: "category_1".to_string(),
                    entry_id: 1,
                    votes: votes.clone(),
                },
                &vec![],
            )
            .unwrap();

            let res: Votes = app
                .wrap()
                .query_wasm_smart(
                    contract_addr,
                    &QueryMsg::Votes {
                        entry_id: 1,
                        maker_addr: SECOND_MAKER.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(res, votes);
        }

        #[test]
        fn test_invalid_maker() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let makers_cw4_group = setup_cw4_group(&mut app, vec![FIRST_MAKER]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                makers_cw4_group.to_string(),
            );

            setup_category(&mut app, contract_addr.clone(), "category_1".to_string());
            setup_entry(
                &mut app,
                contract_addr.clone(),
                "entry_name".to_string(),
                "category_1".to_string(),
                FIRST_MAKER.to_string(),
                "maker_name".to_string(),
                "entry_breeder".to_string(),
                "entry_genetics".to_string(),
                "entry_farmer".to_string(),
            );

            let votes = Votes {
                look: Uint128::new(775),
                smell: Uint128::new(820),
                taste: Uint128::new(1000),
                post_melt: Uint128::new(250),
            };

            let err = app
                .execute_contract(
                    Addr::unchecked(FIRST_MAKER),
                    contract_addr.clone(),
                    &ExecuteMsg::Vote {
                        category: "category_1".to_string(),
                        entry_id: 1,
                        votes: votes.clone(),
                    },
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidMaker {}.to_string()
            );

            let err = app
                .execute_contract(
                    Addr::unchecked(SECOND_MAKER),
                    contract_addr.clone(),
                    &ExecuteMsg::Vote {
                        category: "category_1".to_string(),
                        entry_id: 1,
                        votes: votes.clone(),
                    },
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_invalid_category() {
            let mut app = mock_app();
            let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
            let makers_cw4_group = setup_cw4_group(&mut app, vec![FIRST_MAKER, SECOND_MAKER]);
            let contract_addr = setup_contract(
                &mut app,
                admin_cw4_group.to_string(),
                makers_cw4_group.to_string(),
            );

            setup_category(&mut app, contract_addr.clone(), "category_1".to_string());
            setup_entry(
                &mut app,
                contract_addr.clone(),
                "entry_name".to_string(),
                "category_1".to_string(),
                FIRST_MAKER.to_string(),
                "maker_name".to_string(),
                "entry_breeder".to_string(),
                "entry_genetics".to_string(),
                "entry_farmer".to_string(),
            );

            let votes = Votes {
                look: Uint128::new(775),
                smell: Uint128::new(820),
                taste: Uint128::new(1000),
                post_melt: Uint128::new(250),
            };

            let err = app
                .execute_contract(
                    Addr::unchecked(SECOND_MAKER),
                    contract_addr.clone(),
                    &ExecuteMsg::Vote {
                        category: "category_2".to_string(),
                        entry_id: 1,
                        votes: votes.clone(),
                    },
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidCategory {}.to_string()
            );
        }
    }
}

mod query {
    use crate::msg::TallyVotesResponse;

    use super::*;

    #[test]
    fn test_categories() {
        let mut app = mock_app();
        let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
        let makers_cw4_group = setup_cw4_group(&mut app, vec![FIRST_MAKER, SECOND_MAKER]);
        let contract_addr = setup_contract(
            &mut app,
            admin_cw4_group.to_string(),
            makers_cw4_group.to_string(),
        );

        setup_category(&mut app, contract_addr.clone(), "category_1".to_string());
        setup_category(&mut app, contract_addr.clone(), "category_2".to_string());
        setup_category(&mut app, contract_addr.clone(), "category_3".to_string());

        let res: Vec<String> = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Categories {})
            .unwrap();
        assert_eq!(res, vec!["category_1", "category_2", "category_3"]);
    }

    #[test]
    fn test_entries() {
        let mut app = mock_app();
        let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
        let makers_cw4_group = setup_cw4_group(&mut app, vec![FIRST_MAKER, SECOND_MAKER]);
        let contract_addr = setup_contract(
            &mut app,
            admin_cw4_group.to_string(),
            makers_cw4_group.to_string(),
        );

        setup_category(&mut app, contract_addr.clone(), "category_1".to_string());
        setup_entry(
            &mut app,
            contract_addr.clone(),
            "entry_name".to_string(),
            "category_1".to_string(),
            FIRST_MAKER.to_string(),
            "maker_name".to_string(),
            "entry_breeder".to_string(),
            "entry_genetics".to_string(),
            "entry_farmer".to_string(),
        );
        setup_entry(
            &mut app,
            contract_addr.clone(),
            "entry_name".to_string(),
            "category_1".to_string(),
            SECOND_MAKER.to_string(),
            "maker_name".to_string(),
            "entry_breeder".to_string(),
            "entry_genetics".to_string(),
            "entry_farmer".to_string(),
        );
        setup_entry(
            &mut app,
            contract_addr.clone(),
            "entry_name".to_string(),
            "category_1".to_string(),
            "third_maker".to_string(),
            "maker_name".to_string(),
            "entry_breeder".to_string(),
            "entry_genetics".to_string(),
            "entry_farmer".to_string(),
        );

        let res: Vec<EntriesResponse> = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Entries {
                    category: "category_1".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(
            res[1].data,
            Entry {
                name: "entry_name".to_string(),
                category: "category_1".to_string(),
                maker_addr: Addr::unchecked(SECOND_MAKER.to_string()),
                maker_name: "maker_name".to_string(),
                breeder: "entry_breeder".to_string(),
                genetics: "entry_genetics".to_string(),
                farmer: "entry_farmer".to_string(),
            }
        );

        let res: Vec<EntriesResponse> = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Entries {
                    category: "category_1".to_string(),
                    start_after: Some(2),
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].data,
            Entry {
                name: "entry_name".to_string(),
                category: "category_1".to_string(),
                maker_addr: Addr::unchecked("third_maker".to_string()),
                maker_name: "maker_name".to_string(),
                breeder: "entry_breeder".to_string(),
                genetics: "entry_genetics".to_string(),
                farmer: "entry_farmer".to_string(),
            }
        );

        let res: Vec<EntriesResponse> = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Entries {
                    category: "category_1".to_string(),
                    start_after: Some(1),
                    limit: Some(1),
                },
            )
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].data,
            Entry {
                name: "entry_name".to_string(),
                category: "category_1".to_string(),
                maker_addr: Addr::unchecked(SECOND_MAKER.to_string()),
                maker_name: "maker_name".to_string(),
                breeder: "entry_breeder".to_string(),
                genetics: "entry_genetics".to_string(),
                farmer: "entry_farmer".to_string(),
            }
        )
    }

    #[test]
    fn test_votes() {
        let mut app = mock_app();
        let admin_cw4_group = setup_cw4_group(&mut app, vec![ADMIN]);
        let makers_cw4_group = setup_cw4_group(
            &mut app,
            vec![
                FIRST_MAKER,
                SECOND_MAKER,
                "third_maker",
                "fourth_maker",
                "fifth_maker",
            ],
        );
        let contract_addr = setup_contract(
            &mut app,
            admin_cw4_group.to_string(),
            makers_cw4_group.to_string(),
        );

        setup_category(&mut app, contract_addr.clone(), "category_1".to_string());
        setup_entry(
            &mut app,
            contract_addr.clone(),
            "entry_name".to_string(),
            "category_1".to_string(),
            FIRST_MAKER.to_string(),
            "maker_name".to_string(),
            "entry_breeder".to_string(),
            "entry_genetics".to_string(),
            "entry_farmer".to_string(),
        );
        setup_vote(
            &mut app,
            contract_addr.clone(),
            SECOND_MAKER,
            "category_1".to_string(),
            1,
            Votes {
                look: Uint128::new(775),
                smell: Uint128::new(820),
                taste: Uint128::new(1000),
                post_melt: Uint128::new(250),
            },
        );
        setup_vote(
            &mut app,
            contract_addr.clone(),
            "third_maker",
            "category_1".to_string(),
            1,
            Votes {
                look: Uint128::new(450),
                smell: Uint128::new(259),
                taste: Uint128::new(720),
                post_melt: Uint128::new(180),
            },
        );
        setup_vote(
            &mut app,
            contract_addr.clone(),
            "fourth_maker",
            "category_1".to_string(),
            1,
            Votes {
                look: Uint128::new(603),
                smell: Uint128::new(278),
                taste: Uint128::new(383),
                post_melt: Uint128::new(286),
            },
        );
        setup_vote(
            &mut app,
            contract_addr.clone(),
            "fifth_maker",
            "category_1".to_string(),
            1,
            Votes {
                look: Uint128::new(950),
                smell: Uint128::new(279),
                taste: Uint128::new(632),
                post_melt: Uint128::new(492),
            },
        );

        let res: TallyVotesResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::TallyVotes {
                    entry_id: 1,
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(res.votes.len(), 4);
        assert_eq!(res.votes[0].maker_addr, "fifth_maker");
        assert_eq!(res.votes[0].sum, Uint128::new(2353));
        assert_eq!(res.votes[1].maker_addr, "fourth_maker");
        assert_eq!(res.votes[1].sum, Uint128::new(1550));
        assert_eq!(res.sum.look, Uint128::new(2778));
        assert_eq!(res.sum.smell, Uint128::new(1636));
        assert_eq!(res.sum.taste, Uint128::new(2735));
        assert_eq!(res.sum.post_melt, Uint128::new(1208));

        let res: TallyVotesResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::TallyVotes {
                    entry_id: 1,
                    start_after: Some("fourth_maker".to_string()),
                    limit: Some(2),
                },
            )
            .unwrap();
        assert_eq!(res.votes.len(), 2);
        assert_eq!(res.votes[0].maker_addr, SECOND_MAKER);
        assert_eq!(res.votes[0].sum, Uint128::new(2845));
        assert_eq!(res.votes[1].maker_addr, "third_maker");
        assert_eq!(res.votes[1].sum, Uint128::new(1609));
        assert_eq!(res.sum.look, Uint128::new(1225));
        assert_eq!(res.sum.smell, Uint128::new(1079));
        assert_eq!(res.sum.taste, Uint128::new(1720));
        assert_eq!(res.sum.post_melt, Uint128::new(430));
    }
}
