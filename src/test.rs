#[cfg(test)]
mod tests {
    use crate::{
        error::ContractError,
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        state::DrawState,
    };
    use coreum_test_tube::{
        Account, AssetFT, Bank, CoreumTestApp, Module, SigningAccount, Staking, Wasm,
    };
    use coreum_wasm_sdk::types::cosmos::bank::v1beta1::{MsgSend, QueryBalanceRequest};
    use coreum_wasm_sdk::types::cosmos::staking::v1beta1::{
        CommissionRates, Description, MsgCreateValidator,
    };

    use bech32::{Bech32, Hrp};

    use coreum_wasm_sdk::types::coreum::asset::ft::v1::{
        MsgMint, QueryBalanceRequest as FtQueryBalanceRequest,
        QueryBalanceResponse as FtQueryBalanceResponse,
    };

    use coreum_wasm_sdk::shim::Any;
    use coreum_wasm_sdk::types::cosmos::base::v1beta1::Coin as BaseCoin;
    use cosmrs::proto;
    use cosmwasm_std::{coin, Coin as CosmoCoin, Uint128};
    use prost::Message;
    use ring::{
        rand,
        signature::{self, KeyPair},
    };

    const FEE_DENOM: &str = "ucore";
    const TICKET_TOKEN: &str = "TICKET";
    const TICKET_PRECISION: u32 = 6;
    const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
    const UNDELEGATION_DAYS: u64 = 7;
    const TICKET_PRICE: u128 = 200_000_000; //200 COREUM

    fn get_validator_address(address: &str) -> String {
        let (_, data) = bech32::decode(address).expect("failed to decode");
        let val_hrp = Hrp::parse("corevaloper").unwrap();
        bech32::encode::<Bech32>(val_hrp, &data).expect("failed to encode string")
    }

    fn get_ed25519_pubkey() -> Any {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
        let public_key = key_pair.public_key();
        Any {
            type_url: "/cosmos.crypto.ed25519.PubKey".to_string(),
            value: proto::cosmos::crypto::ed25519::PubKey {
                key: public_key.as_ref().to_vec(),
            }
            .encode_to_vec(),
        }
    }

    fn create_validator(app: &CoreumTestApp, signer: &SigningAccount) -> String {
        let staking: Staking<'_, CoreumTestApp> = Staking::new(app);
        let delegator_address = signer.address();
        // Convert core1... to corevaloper1...
        let validator_address: String = get_validator_address(&delegator_address);

        let msg = MsgCreateValidator {
            #[allow(deprecated)]
            description: Some(Description {
                moniker: "moniker".to_string(),
                identity: "".to_string(),
                website: "".to_string(),
                security_contact: "".to_string(),
                details: "".to_string(),
            }),
            commission: Some(CommissionRates {
                rate: "1".to_string(),
                max_rate: "5".to_string(),
                max_change_rate: "1".to_string(),
            }),
            min_self_delegation: "20000000000".to_string(),
            delegator_address: signer.address().to_string(),
            validator_address: validator_address.clone(),
            pubkey: Some(get_ed25519_pubkey()),
            value: Some(BaseCoin {
                denom: FEE_DENOM.to_string(),
                amount: "21000000000".to_string(),
            }),
        };

        println!("Signer address: {}", signer.address());
        staking.create_validator(msg, signer).unwrap();
        validator_address
    }

    fn store_and_instantiate(
        wasm: &Wasm<'_, CoreumTestApp>,
        admin: &SigningAccount,
        validator_address: String,
        total_tickets: Uint128,
        ticket_price: Uint128,
        max_tickets_per_user: Uint128,
    ) -> String {
        let wasm_byte_code = std::fs::read("artifacts/coreum_fun_contract.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &admin)
            .unwrap()
            .data
            .code_id;

        wasm.instantiate(
            code_id,
            &InstantiateMsg {
                ticket_token_symbol: TICKET_TOKEN.to_string(),
                core_denom: FEE_DENOM.to_string(),
                validator_address,
                total_tickets,
                ticket_price,
                max_tickets_per_user,
            },
            None,
            "coreum-fun".into(),
            &[coin(10_000_000, FEE_DENOM)], // Add 10 CORE (10_000_000 ucore) as initial funds
            &admin,
        )
        .unwrap()
        .data
        .address
    }

    #[test]
    fn contract_instantiation() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        // Force account registration
        let bank = Bank::new(&app);
        bank.send(
            MsgSend {
                from_address: validator_creator.address(),
                to_address: admin.address(),
                amount: vec![BaseCoin {
                    amount: 1u128.to_string(),
                    denom: FEE_DENOM.to_string(),
                }],
            },
            &validator_creator,
        )
        .unwrap();

        let wasm: Wasm<'_, CoreumTestApp> = Wasm::new(&app);
        // let validator_operator_address = create_validator(&app, &validator_creator);

        let validator_address: String =
            get_validator_address(validator_creator.address().to_string().as_str());
        println!("Validator creator address: {}", validator_creator.address());
        println!("Validator address: {}", validator_address);
        println!("Admin address: {}", admin.address());

        let validator_address = create_validator(&app, &validator_creator);

        // Instantiate contract
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),     // total_tickets
            Uint128::from(TICKET_PRICE), // ticket_price
            Uint128::from(10u128),       // max_tickets_per_user
        );

        // Query current state
        let current_state: crate::msg::CurrentStateResponse = wasm
            .query(&contract_address, &QueryMsg::GetCurrentState {})
            .unwrap();

        assert_eq!(current_state.state, DrawState::TicketSalesOpen);
    }

    #[test]
    fn buy_tickets() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000, FEE_DENOM)])
            .unwrap();
        let user: SigningAccount = app
            .init_account(&[coin(100_000_000_000, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000, FEE_DENOM)])
            .unwrap();

        println!("Validator creator address: {}", validator_creator.address());
        println!("Admin address: {}", admin.address());
        println!("User address: {}", user.address());

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Instantiate contract
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),     // total_tickets
            Uint128::from(TICKET_PRICE), // ticket_price
            Uint128::from(10u128),       // max_tickets_per_user
        );

        // Buy tickets
        let number_of_tickets = Uint128::from(5u128);
        let payment: Uint128 = number_of_tickets * Uint128::from(TICKET_PRICE);

        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user,
        )
        .unwrap();

        // Query user's tickets
        let user_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user.address(),
                },
            )
            .unwrap();

        assert_eq!(user_tickets.tickets, number_of_tickets);

        // Query total tickets sold
        let tickets_sold: crate::msg::TicketsSoldResponse = wasm
            .query(&contract_address, &QueryMsg::GetNumberOfTicketsSold {})
            .unwrap();

        assert_eq!(tickets_sold.tickets_sold, number_of_tickets);
    }

    #[test]
    fn select_winner_and_burn_tickets() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Instantiate contract
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),     // total_tickets
            Uint128::from(TICKET_PRICE), // ticket_price
            Uint128::from(1000u128),     // max_tickets_per_user (1000 tickets just to test)
        );

        // Buy tickets
        let number_of_tickets = Uint128::from(1000u128);
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE); // 200,000,000 ucore * 1000 tickets = 200,000,000,000 ucore
        println!("payment: {}", payment);
        let ticket_denom = format!("u{}-{}", TICKET_TOKEN.to_lowercase(), contract_address);

        //before selecting the winner, we need to have sold the to number_of_tickets of tickets sold or set the draw state to tickets_sold_out_accumulation_in_progress
        //Otherwise, we will get an error:  ExecuteError { msg: "failed to execute message; message index: 0: Invalid draw state (expected: TicketsSoldOutAccumulationInProgress, actual: TicketSalesOpen): execute wasm contract failed" }
        // let's sell the total number_of_tickets of tickets
        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user,
        )
        .unwrap();

        //check the contract balance
        let contract_balance: crate::msg::BalanceResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::Balance {
                    account: contract_address.clone(),
                },
            )
            .unwrap();
        println!("contract_balance: {}", contract_balance.balance);

        // Add bonus rewards
        let bonus_amount = Uint128::from(1000000u128);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::AddBonusRewardToThePool {
                amount: bonus_amount,
            },
            &[coin(bonus_amount.u128(), FEE_DENOM)],
            &admin,
        )
        .unwrap();

        //query the current draw state: should be TicketsSoldOutAccumulationInProgress because we have sold the total number_of_tickets of tickets
        // let current_draw_state: crate::msg::CurrentStateResponse = wasm
        //     .query(&contract_address, &QueryMsg::GetCurrentState {})
        //     .unwrap();
        // assert_eq!(current_draw_state.state, DrawState::TicketSalesOpen);

        // wasm.execute(
        //     &contract_address,
        //     &ExecuteMsg::BuyTicket { number_of_tickets },
        //     &[coin(payment.u128(), FEE_DENOM)],
        //     &user,
        // )
        // .unwrap();

        // Verify tickets were received
        let user_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user.address(),
                },
            )
            .unwrap();
        assert_eq!(user_tickets.tickets, number_of_tickets);

        // Check the delegate amount before undelegation
        let contract_delegated_tokens: crate::msg::DelegatedAmountResponse = wasm
            .query(&contract_address, &QueryMsg::GetDelegatedAmount {})
            .unwrap();
        println!(
            "contract_delegated_tokens before undelegation: {:?}",
            contract_delegated_tokens
        );

        // Select winner and complete undelegation
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SelectWinnerAndUndelegate {
                winner_address: user.address(),
            },
            &[],
            &admin,
        )
        .unwrap();

        // Wait for undelegation period to complete
        app.increase_time(SECONDS_PER_DAY * UNDELEGATION_DAYS + 10000000);

        // Add bonus rewards
        let bonus_amount = Uint128::from(1000000u128); // 1 COREUM
        wasm.execute(
            &contract_address,
            &ExecuteMsg::AddBonusRewardToThePool {
                amount: bonus_amount,
            },
            &[coin(bonus_amount.u128(), FEE_DENOM)],
            &admin,
        )
        .unwrap();

        // Check contract state and balances before sending funds
        let state: crate::msg::CurrentStateResponse = wasm
            .query(&contract_address, &QueryMsg::GetCurrentState {})
            .unwrap();
        println!("Contract state before sending funds: {:?}", state.state);

        let delegated: crate::msg::DelegatedAmountResponse = wasm
            .query(&contract_address, &QueryMsg::GetDelegatedAmount {})
            .unwrap();
        println!("Delegated amount before sending funds: {:?}", delegated);

        let bank = Bank::new(&app);
        let contract_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: contract_address.clone(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap();
        println!(
            "Contract balance before sending funds: {:?}",
            contract_balance
        );

        // Send funds to winner
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SendFundsToWinner {},
            &[],
            &admin,
        )
        .unwrap();

        // Query winner
        let winner: crate::msg::WinnerResponse = wasm
            .query(&contract_address, &QueryMsg::GetWinner {})
            .unwrap();

        assert_eq!(winner.winner, Some(user.address()));

        //here we need to advance the block time to let the app know that the accumulation period is over.
        //otherwise, we will get an error: Undelegation period not completed (current block: 10, undelegation block: 302410)
        // let's set the block time to 10 seconds

        //check the contract delegated tokens: should be equal to the number of tickets sold * ticket price
        let contract_delegated_tokens: crate::msg::DelegatedAmountResponse = wasm
            .query(&contract_address, &QueryMsg::GetDelegatedAmount {})
            .unwrap();
        println!("contract_delegated_tokens: {:?}", contract_delegated_tokens);

        let undelegation_period_seconds: u64 = SECONDS_PER_DAY * UNDELEGATION_DAYS + 10000;
        let current_timestamp = app.get_block_timestamp();
        let target_timestamp = current_timestamp.seconds() + undelegation_period_seconds;

        println!("Current timestamp: {}", current_timestamp);
        app.increase_time(undelegation_period_seconds);

        println!("Target timestamp: {}", target_timestamp);
        println!("New timestamp: {}", app.get_block_timestamp());

        //check the contract delegated tokens: should be 0 ucore now
        let contract_delegated_tokens: crate::msg::DelegatedAmountResponse = wasm
            .query(&contract_address, &QueryMsg::GetDelegatedAmount {})
            .unwrap();
        println!("contract_delegated_tokens: {:?}", contract_delegated_tokens);

        // Burn tickets

        let tickets_to_burn = CosmoCoin {
            amount: number_of_tickets * Uint128::from(10u128).pow(TICKET_PRECISION),
            denom: ticket_denom,
        };

        wasm.execute(
            &contract_address,
            &ExecuteMsg::BurnTickets { number_of_tickets },
            &[tickets_to_burn],
            &user,
        )
        .unwrap();

        // Query total tickets burned
        let total_burned: crate::msg::TotalBurnedResponse = wasm
            .query(&contract_address, &QueryMsg::GetTotalTicketsBurned {})
            .unwrap();

        assert_eq!(total_burned.total_burned, number_of_tickets);
    }

    #[test]
    fn test_error_cases() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Test invalid instantiation
        let result = wasm.instantiate(
            1, // invalid code_id
            &InstantiateMsg {
                ticket_token_symbol: TICKET_TOKEN.to_string(),
                validator_address: validator_address.clone(),
                total_tickets: Uint128::zero(), // invalid total_tickets
                ticket_price: Uint128::from(1000000u128),
                max_tickets_per_user: Uint128::from(10u128),
                core_denom: FEE_DENOM.to_string(),
            },
            None,
            "coreum-fun".into(),
            &[],
            &admin,
        );
        assert!(result.is_err());

        // Instantiate contract properly
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(1000000u128),
            Uint128::from(10u128),
        );

        // Test buying tickets with insufficient funds
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket {
                number_of_tickets: Uint128::from(5u128),
            },
            &[coin(1000u128, FEE_DENOM)], // insufficient funds
            &user,
        );
        assert!(result.is_err());

        // Test unauthorized winner selection
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::SelectWinnerAndUndelegate {
                winner_address: user.address(),
            },
            &[],
            &user, // not the owner
        );
        assert!(result.is_err());

        // Test burning tickets before winner selection
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::BurnTickets {
                number_of_tickets: Uint128::from(5u128),
            },
            &[],
            &user,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_bonus_rewards() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(1000u128),
        );

        // Add bonus rewards
        let bonus_amount = Uint128::from(1000000u128);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::AddBonusRewardToThePool {
                amount: bonus_amount,
            },
            &[coin(bonus_amount.u128(), FEE_DENOM)],
            &admin,
        )
        .unwrap();

        // Verify bonus rewards were added
        let bonus_rewards: crate::msg::BonusRewardsResponse = wasm
            .query(&contract_address, &QueryMsg::GetBonusRewards {})
            .unwrap();
        assert_eq!(bonus_rewards.bonus_rewards, bonus_amount);
    }

    #[test]
    fn test_ticket_burning_edge_cases() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(1000u128),
        );

        // Test burning before winner selection
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::BurnTickets {
                number_of_tickets: Uint128::from(5u128),
            },
            &[],
            &user,
        );
        assert!(result.is_err());

        // Buy tickets first
        let number_of_tickets = Uint128::from(10u128);
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user,
        )
        .unwrap();

        // Test burning more tickets than owned
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::BurnTickets {
                number_of_tickets: Uint128::from(20u128),
            },
            &[],
            &user,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_state_transitions() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(1000u128),
        );

        // Buy all tickets to trigger state change
        let number_of_tickets = Uint128::from(1000u128);
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user,
        )
        .unwrap();

        // Verify state changed
        let state: crate::msg::CurrentStateResponse = wasm
            .query(&contract_address, &QueryMsg::GetCurrentState {})
            .unwrap();
        assert_eq!(state.state, DrawState::TicketsSoldOutAccumulationInProgress);

        //Manually set the state to DrawFinished
        wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateDrawState {
                new_state: DrawState::DrawFinished,
            },
            &[],
            &admin,
        )
        .unwrap();

        // Verify state changed
        let state: crate::msg::CurrentStateResponse = wasm
            .query(&contract_address, &QueryMsg::GetCurrentState {})
            .unwrap();
        assert_eq!(state.state, DrawState::DrawFinished);
    }

    #[test]
    fn test_user_limits() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user1 = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user2 = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(100u128), // max 100 tickets per user
        );

        // Test max tickets per user
        let number_of_tickets = Uint128::from(101u128);
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE);
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user1,
        );
        assert!(result.is_err());

        // Test multiple users buying tickets
        let tickets_user1 = Uint128::from(50u128);
        let tickets_user2 = Uint128::from(50u128);

        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket {
                number_of_tickets: tickets_user1,
            },
            &[coin(tickets_user1.u128() * TICKET_PRICE, FEE_DENOM)],
            &user1,
        )
        .unwrap();

        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket {
                number_of_tickets: tickets_user2,
            },
            &[coin(tickets_user2.u128() * TICKET_PRICE, FEE_DENOM)],
            &user2,
        )
        .unwrap();

        // Verify ticket distribution
        let user1_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user1.address(),
                },
            )
            .unwrap();
        assert_eq!(user1_tickets.tickets, tickets_user1);

        let user2_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user2.address(),
                },
            )
            .unwrap();
        assert_eq!(user2_tickets.tickets, tickets_user2);
    }

    #[test]
    fn test_rewards_distribution() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(1000u128),
        );

        // Buy tickets
        let number_of_tickets = Uint128::from(1000u128);
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user,
        )
        .unwrap();

        // Add bonus rewards
        let bonus_amount = Uint128::from(1000000u128);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::AddBonusRewardToThePool {
                amount: bonus_amount,
            },
            &[coin(bonus_amount.u128(), FEE_DENOM)],
            &admin,
        )
        .unwrap();

        // Get initial user balance
        let bank = Bank::new(&app);
        let initial_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: user.address(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        // Select winner
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SelectWinnerAndUndelegate {
                winner_address: user.address(),
            },
            &[],
            &admin,
        )
        .unwrap();

        // Advance time to complete undelegation
        app.increase_time(SECONDS_PER_DAY * UNDELEGATION_DAYS + 1000);

        // Send funds to winner
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SendFundsToWinner {},
            &[],
            &admin,
        )
        .unwrap();

        // Get final user balance
        let final_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: user.address(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        // Calculate received rewards
        let received_rewards = final_balance - initial_balance;

        // Get expected rewards from contract
        let winner: crate::msg::WinnerResponse = wasm
            .query(&contract_address, &QueryMsg::GetWinner {})
            .unwrap();

        // Verify winner address
        assert_eq!(winner.winner, Some(user.address()));

        // Verify rewards amount
        // The received rewards should be equal to the accumulated rewards + bonus rewards
        assert_eq!(
            Uint128::from(received_rewards),
            winner.rewards,
            "Expected rewards: {}, Received rewards: {}",
            winner.rewards,
            received_rewards
        );

        // Verify the rewards include both accumulated and bonus rewards

        // Get accumulated rewards at undelegation
        let accumulated_rewards_at_undelegation: crate::msg::AccumulatedRewardsAtUndelegationResponse = wasm
            .query(&contract_address, &QueryMsg::GetAccumulatedRewardsAtUndelegation {})
            .unwrap();

        let bonus_rewards: crate::msg::BonusRewardsResponse = wasm
            .query(&contract_address, &QueryMsg::GetBonusRewards {})
            .unwrap();

        assert_eq!(
            winner.rewards,
            accumulated_rewards_at_undelegation.accumulated_rewards + bonus_rewards.bonus_rewards,
            "Total rewards should be sum of accumulated and bonus rewards"
        );
    }

    #[test]
    fn test_burn_tickets_and_refund() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let bank = Bank::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(1000u128),
        );

        // Buy tickets (we need to buy all tickets to trigger the state transition)
        let number_of_tickets = Uint128::from(1000u128);
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE);

        // Get initial balance
        let initial_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: user.address(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user,
        )
        .unwrap();

        // Verify tickets were received
        let user_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user.address(),
                },
            )
            .unwrap();
        assert_eq!(user_tickets.tickets, number_of_tickets);

        // Check the delegate amount before undelegation
        let contract_delegated_tokens: crate::msg::DelegatedAmountResponse = wasm
            .query(&contract_address, &QueryMsg::GetDelegatedAmount {})
            .unwrap();
        println!(
            "contract_delegated_tokens before undelegation: {:?}",
            contract_delegated_tokens
        );

        // Select winner and complete undelegation
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SelectWinnerAndUndelegate {
                winner_address: user.address(),
            },
            &[],
            &admin,
        )
        .unwrap();

        // Wait for undelegation period to complete
        app.increase_time(SECONDS_PER_DAY * UNDELEGATION_DAYS + 10000000);

        // Add bonus rewards
        let bonus_amount = Uint128::from(1000000u128); // 1 COREUM
        wasm.execute(
            &contract_address,
            &ExecuteMsg::AddBonusRewardToThePool {
                amount: bonus_amount,
            },
            &[coin(bonus_amount.u128(), FEE_DENOM)],
            &admin,
        )
        .unwrap();

        // Check contract state and balances before sending funds
        let state: crate::msg::CurrentStateResponse = wasm
            .query(&contract_address, &QueryMsg::GetCurrentState {})
            .unwrap();
        println!("Contract state before sending funds: {:?}", state.state);

        let delegated: crate::msg::DelegatedAmountResponse = wasm
            .query(&contract_address, &QueryMsg::GetDelegatedAmount {})
            .unwrap();
        println!("Delegated amount before sending funds: {:?}", delegated);

        let bank = Bank::new(&app);
        let contract_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: contract_address.clone(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap();
        println!(
            "Contract balance before sending funds: {:?}",
            contract_balance
        );

        // Send funds to winner
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SendFundsToWinner {},
            &[],
            &admin,
        )
        .unwrap();

        // Burn tickets
        let ticket_denom = format!("u{}-{}", TICKET_TOKEN.to_lowercase(), contract_address);
        let tickets_to_burn = CosmoCoin {
            amount: number_of_tickets * Uint128::from(10u128).pow(TICKET_PRECISION),
            denom: ticket_denom.clone(),
        };

        // Get the user ticket balance before burning tickets
        let initial_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: user.address(),
                denom: ticket_denom.clone(),
            })
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        assert_eq!(
            initial_balance,
            (number_of_tickets * Uint128::from(10u128).pow(TICKET_PRECISION)).u128()
        );

        wasm.execute(
            &contract_address,
            &ExecuteMsg::BurnTickets { number_of_tickets },
            &[tickets_to_burn],
            &user,
        )
        .unwrap();

        // Verify no tickets left (user balance)
        let final_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: user.address(),
                denom: ticket_denom.clone(),
            })
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        assert_eq!(final_balance, 0);

        // Verify no tickets left (contract state)
        let user_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user.address(),
                },
            )
            .unwrap();
        assert_eq!(user_tickets.tickets, Uint128::zero());

        // Verify refund was received
        let final_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: user.address(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        // The final balance should be close to initial balance (minus gas fees)
        let expected_refund = payment.u128();
        let actual_refund = final_balance - initial_balance;
        assert!(
            actual_refund >= expected_refund * 99 / 100, // Allow for 1% gas fee
            "Expected refund: {}, Actual refund: {}",
            expected_refund,
            actual_refund
        );
    }

    #[test]
    fn test_ticket_token_balance() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let bank = Bank::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(1000u128),
        );

        // Buy tickets
        let number_of_tickets = Uint128::from(5u128);
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE);

        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket { number_of_tickets },
            &[coin(payment.u128(), FEE_DENOM)],
            &user,
        )
        .unwrap();

        // Get ticket token denom
        let ticket_denom = format!("u{}-{}", TICKET_TOKEN.to_lowercase(), contract_address);

        // Query ticket token balance
        let ticket_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: user.address(),
                denom: ticket_denom.clone(),
            })
            .unwrap()
            .balance
            .unwrap();

        // Verify denom
        assert_eq!(ticket_balance.denom, ticket_denom);

        // Verify amount (should be number_of_tickets * 10^6)
        let expected_amount =
            (number_of_tickets * Uint128::from(10u128).pow(TICKET_PRECISION)).to_string();
        assert_eq!(ticket_balance.amount, expected_amount);

        // Verify through contract query as well
        let user_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user.address(),
                },
            )
            .unwrap();
        assert_eq!(user_tickets.tickets, number_of_tickets);
    }

    #[test]
    fn test_ticket_purchase_limit() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let bank = Bank::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Set max tickets per user to 3
        let max_tickets = Uint128::from(3u128);
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            max_tickets,
        );

        // Buy tickets one by one
        for i in 1u32..=3 {
            let number_of_tickets = Uint128::from(1u128);
            let payment = number_of_tickets * Uint128::from(TICKET_PRICE);

            wasm.execute(
                &contract_address,
                &ExecuteMsg::BuyTicket { number_of_tickets },
                &[coin(payment.u128(), FEE_DENOM)],
                &user,
            )
            .unwrap();

            // Verify ticket count after each purchase
            let user_tickets: crate::msg::UserTicketsResponse = wasm
                .query(
                    &contract_address,
                    &QueryMsg::GetUserNumberOfTickets {
                        address: user.address(),
                    },
                )
                .unwrap();
            assert_eq!(user_tickets.tickets, Uint128::from(i as u128));
        }

        // Try to buy one more ticket (should fail)
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket {
                number_of_tickets: Uint128::from(1u128),
            },
            &[coin(TICKET_PRICE, FEE_DENOM)],
            &user,
        );
        assert!(result.is_err());

        // Verify final ticket count
        let user_tickets: crate::msg::UserTicketsResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetUserNumberOfTickets {
                    address: user.address(),
                },
            )
            .unwrap();
        assert_eq!(user_tickets.tickets, max_tickets);
    }

    #[test]
    fn test_query_ticket_holders() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user1 = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let user2 = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(100u128),
        );

        // Buy tickets for user1
        let tickets_user1 = Uint128::from(50u128);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket {
                number_of_tickets: tickets_user1,
            },
            &[coin(tickets_user1.u128() * TICKET_PRICE, FEE_DENOM)],
            &user1,
        )
        .unwrap();

        // Buy tickets for user2
        let tickets_user2 = Uint128::from(30u128);
        wasm.execute(
            &contract_address,
            &ExecuteMsg::BuyTicket {
                number_of_tickets: tickets_user2,
            },
            &[coin(tickets_user2.u128() * TICKET_PRICE, FEE_DENOM)],
            &user2,
        )
        .unwrap();

        // Query ticket holders
        let holders: crate::msg::TicketHoldersResponse = wasm
            .query(&contract_address, &QueryMsg::GetTicketHolders {})
            .unwrap();

        println!("Total holders: {}", holders.total_holders);
        for holder in &holders.holders {
            println!(
                "Holder: {}, Tickets: {}, Win Chance: {}",
                holder.address, holder.tickets, holder.win_chance
            );
        }

        // Verify results
        assert_eq!(holders.total_holders, 2);
        assert_eq!(holders.holders.len(), 2);

        // Verify user1's tickets
        let user1_holder = holders
            .holders
            .iter()
            .find(|h| h.address == user1.address())
            .unwrap();
        assert_eq!(user1_holder.tickets, tickets_user1);

        // Verify user2's tickets
        let user2_holder = holders
            .holders
            .iter()
            .find(|h| h.address == user2.address())
            .unwrap();
        assert_eq!(user2_holder.tickets, tickets_user2);

        // Add test for ticket transfer
        let user3 = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        // Transfer 10 tickets from user1 to user3
        let transfer_amount = Uint128::from(10u128);
        let ticket_denom = format!("u{}-{}", TICKET_TOKEN.to_lowercase(), contract_address);

        let bank = Bank::new(&app);
        bank.send(
            MsgSend {
                from_address: user1.address(),
                to_address: user3.address(),
                amount: vec![BaseCoin {
                    amount: (transfer_amount * Uint128::from(10u128).pow(TICKET_PRECISION))
                        .to_string(),
                    denom: ticket_denom,
                }],
            },
            &user1,
        )
        .unwrap();

        // Query ticket holders again
        let holders: crate::msg::TicketHoldersResponse = wasm
            .query(&contract_address, &QueryMsg::GetTicketHolders {})
            .unwrap();

        println!("\nAfter transfer:");
        println!("Total holders: {}", holders.total_holders);
        println!("Number of holders in vector: {}", holders.holders.len());
        for holder in &holders.holders {
            println!(
                "Holder: {}, Tickets: {}, Win Chance: {}",
                holder.address, holder.tickets, holder.win_chance
            );
        }
        println!(
            "All holder addresses: {:?}",
            holders
                .holders
                .iter()
                .map(|h| &h.address)
                .collect::<Vec<_>>()
        );

        // Verify results after transfer
        assert_eq!(holders.total_holders, 3);
        assert_eq!(holders.holders.len(), 3);

        // Verify user3 received the tickets
        let user3_holder = holders
            .holders
            .iter()
            .find(|h| h.address == user3.address())
            .unwrap();
        assert_eq!(user3_holder.tickets, transfer_amount);

        // Verify user1's remaining tickets
        let user1_holder = holders
            .holders
            .iter()
            .find(|h| h.address == user1.address())
            .unwrap();
        assert_eq!(user1_holder.tickets, tickets_user1 - transfer_amount);
    }

    #[test]
    fn test_multiple_buyers_and_burn() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Instantiate contract with 500 tickets
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(500u128),      // total_tickets
            Uint128::from(TICKET_PRICE), // ticket_price
            Uint128::from(1u128),        // max_tickets_per_user (1 ticket per user)
        );

        // Create 500 users and buy one ticket each
        let mut users = Vec::new();
        for i in 0..500 {
            let user = app
                .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
                .unwrap();
            users.push(user);

            // Buy one ticket
            wasm.execute(
                &contract_address,
                &ExecuteMsg::BuyTicket {
                    number_of_tickets: Uint128::from(1u128),
                },
                &[coin(TICKET_PRICE, FEE_DENOM)],
                &users[i],
            )
            .unwrap();
        }

        // Verify total tickets sold
        let tickets_sold: crate::msg::TicketsSoldResponse = wasm
            .query(&contract_address, &QueryMsg::GetNumberOfTicketsSold {})
            .unwrap();
        assert_eq!(tickets_sold.tickets_sold, Uint128::from(500u128));

        // Query ticket holders
        let holders: crate::msg::TicketHoldersResponse = wasm
            .query(&contract_address, &QueryMsg::GetTicketHolders {})
            .unwrap();
        assert_eq!(holders.total_holders, 500);
        assert_eq!(holders.holders.len(), 500);

        // Send 1 COREUM to contract for gas fees
        let bank = Bank::new(&app);
        bank.send(
            MsgSend {
                from_address: admin.address(),
                to_address: contract_address.clone(),
                amount: vec![BaseCoin {
                    amount: "1000000".to_string(), // 1 COREUM (1_000_000 ucore)
                    denom: FEE_DENOM.to_string(),
                }],
            },
            &admin,
        )
        .unwrap();

        // Select winner and start undelegation
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SelectWinnerAndUndelegate {
                winner_address: users[0].address(),
            },
            &[],
            &admin,
        )
        .unwrap();

        // Wait for undelegation period
        app.increase_time(SECONDS_PER_DAY * UNDELEGATION_DAYS + 10000000);

        // Check contract state and balances before sending funds
        let state: crate::msg::CurrentStateResponse = wasm
            .query(&contract_address, &QueryMsg::GetCurrentState {})
            .unwrap();
        println!("Contract state before sending funds: {:?}", state.state);

        let delegated: crate::msg::DelegatedAmountResponse = wasm
            .query(&contract_address, &QueryMsg::GetDelegatedAmount {})
            .unwrap();
        println!("Delegated amount before sending funds: {:?}", delegated);

        let bank = Bank::new(&app);
        let contract_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: contract_address.clone(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap();
        println!(
            "Contract balance before sending funds: {:?}",
            contract_balance
        );

        let accumulated_rewards: crate::msg::AccumulatedRewardsAtUndelegationResponse = wasm
            .query(
                &contract_address,
                &QueryMsg::GetAccumulatedRewardsAtUndelegation {},
            )
            .unwrap();

        println!("accumulated_rewards: {:?}", accumulated_rewards);

        let bonus_rewards: crate::msg::BonusRewardsResponse = wasm
            .query(&contract_address, &QueryMsg::GetBonusRewards {})
            .unwrap();

        let total_rewards = accumulated_rewards.accumulated_rewards + bonus_rewards.bonus_rewards;

        println!("total_rewards: {:?}", total_rewards);

        // Send funds to winner
        wasm.execute(
            &contract_address,
            &ExecuteMsg::SendFundsToWinner {},
            &[],
            &admin,
        )
        .unwrap();

        // Get ticket denom
        let ticket_denom = format!("u{}-{}", TICKET_TOKEN.to_lowercase(), contract_address);

        // Each user burns their ticket
        for user in &users {
            let tickets_to_burn = CosmoCoin {
                amount: Uint128::from(10u128).pow(TICKET_PRECISION),
                denom: ticket_denom.clone(),
            };

            println!("Burning ticket for user: {}", user.address());
            wasm.execute(
                &contract_address,
                &ExecuteMsg::BurnTickets {
                    number_of_tickets: Uint128::from(1u128),
                },
                &[tickets_to_burn],
                user,
            )
            .unwrap();

            // Check holders after each burn
            let holders: crate::msg::TicketHoldersResponse = wasm
                .query(&contract_address, &QueryMsg::GetTicketHolders {})
                .unwrap();
            println!("Holders remaining: {}", holders.total_holders);
        }

        // Verify all tickets are burned
        let total_burned: crate::msg::TotalBurnedResponse = wasm
            .query(&contract_address, &QueryMsg::GetTotalTicketsBurned {})
            .unwrap();
        println!("Total tickets burned: {}", total_burned.total_burned);
        assert_eq!(total_burned.total_burned, Uint128::from(500u128));

        // Verify no tickets left
        let holders: crate::msg::TicketHoldersResponse = wasm
            .query(&contract_address, &QueryMsg::GetTicketHolders {})
            .unwrap();
        println!("Final holders state:");
        println!("Total holders: {}", holders.total_holders);
        println!("Number of holders in vector: {}", holders.holders.len());
        for holder in &holders.holders {
            println!(
                "Holder: {}, Tickets: {}, Win Chance: {}",
                holder.address, holder.tickets, holder.win_chance
            );
        }
        assert_eq!(holders.total_holders, 0);
        assert_eq!(holders.holders.len(), 0);

        // Check that users received their refunds
        for user in &users {
            let balance = bank
                .query_balance(&QueryBalanceRequest {
                    address: user.address(),
                    denom: FEE_DENOM.to_string(),
                })
                .unwrap()
                .balance
                .unwrap()
                .amount
                .parse::<u128>()
                .unwrap();

            // Each user should have their ticket price back (minus gas fees)
            // Initial balance was 100_000_000_000_000_000_000, they spent TICKET_PRICE
            let expected_balance = 100_000_000_000_000_000_000u128 - TICKET_PRICE;
            assert!(
                balance >= expected_balance * 99 / 100, // Allow for 1% gas fee
                "User {} balance {} is less than expected {}",
                user.address(),
                balance,
                expected_balance
            );
        }

        // Check winner's rewards
        let winner_balance = bank
            .query_balance(&QueryBalanceRequest {
                address: users[0].address(),
                denom: FEE_DENOM.to_string(),
            })
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        // Winner should have:
        // 1. Initial balance (100_000_000_000_000_000_000)
        // 2. Minus ticket price (TICKET_PRICE)
        // 3. Plus accumulated rewards
        // 4. Plus bonus rewards
        let expected_winner_balance = 100_000_000_000_000_000_000u128
            + accumulated_rewards.accumulated_rewards.u128()
            + bonus_rewards.bonus_rewards.u128();

        assert!(
            winner_balance >= expected_winner_balance * 99 / 100, // Allow for 1% gas fee
            "Winner balance {} is less than expected {}",
            winner_balance,
            expected_winner_balance
        );
    }

    #[test]
    fn test_token_admin_transfer() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let new_admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Instantiate contract
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(10u128),
        );

        // Transfer token admin
        wasm.execute(
            &contract_address,
            &ExecuteMsg::TransferTokenAdmin {
                new_admin: new_admin.address(),
            },
            &[],
            &admin,
        )
        .unwrap();

        // Verify new admin can mint tokens
        let mint_amount = Uint128::from(1000u128);
        let mint_msg = MsgMint {
            sender: new_admin.address(),
            coin: Some(coreum_wasm_sdk::types::cosmos::base::v1beta1::Coin {
                denom: format!("u{}-{}", TICKET_TOKEN.to_lowercase(), contract_address),
                amount: mint_amount.to_string(),
            }),
            recipient: new_admin.address(),
        };

        // Use Coreum asset module to mint
        let asset = AssetFT::new(&app);
        asset.mint(mint_msg.clone(), &new_admin).unwrap();

        // Verify old admin cannot mint tokens
        let result = asset.mint(mint_msg, &admin);
        assert!(result.is_err());
    }

    #[test]
    fn test_owner_only_functions() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let non_owner = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Instantiate contract
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(10u128),
        );

        // Test execute_select_winner_and_undelegate
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::SelectWinnerAndUndelegate {
                winner_address: non_owner.address(),
            },
            &[],
            &non_owner,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NotOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NotOwner error for select_winner_and_undelegate"
        );

        // Test execute_update_draw_state
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateDrawState {
                new_state: DrawState::DrawFinished,
            },
            &[],
            &non_owner,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NotOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NotOwner error for update_draw_state"
        );

        // Test execute_send_funds
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::SendFundsToWinner {},
            &[],
            &non_owner,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NotOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NotOwner error for send_funds"
        );

        // Test execute_set_undelegation_timestamp
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::SetUndelegationTimestamp {
                timestamp: app.get_block_timestamp().seconds() + 1000,
            },
            &[],
            &non_owner,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NotOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NotOwner error for set_undelegation_timestamp"
        );

        // Test transfer_token_admin
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::TransferTokenAdmin {
                new_admin: non_owner.address(),
            },
            &[],
            &non_owner,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NotOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NotOwner error for transfer_token_admin"
        );

        // Verify that owner can still execute these functions
        wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateDrawState {
                new_state: DrawState::DrawFinished,
            },
            &[],
            &admin,
        )
        .unwrap();

        wasm.execute(
            &contract_address,
            &ExecuteMsg::TransferTokenAdmin {
                new_admin: non_owner.address(),
            },
            &[],
            &admin,
        )
        .unwrap();
    }

    #[test]
    fn test_update_ownership() {
        let app = CoreumTestApp::new();
        let admin = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let new_owner = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let non_owner = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();
        let validator_creator = app
            .init_account(&[coin(100_000_000_000_000_000_000u128, FEE_DENOM)])
            .unwrap();

        let wasm = Wasm::new(&app);
        let validator_address = create_validator(&app, &validator_creator);

        // Instantiate contract
        let contract_address = store_and_instantiate(
            &wasm,
            &admin,
            validator_address,
            Uint128::from(1000u128),
            Uint128::from(TICKET_PRICE),
            Uint128::from(10u128),
        );

        // Test that non-owner cannot transfer ownership
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: new_owner.address(),
                expiry: None,
            }),
            &[],
            &non_owner,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NotOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NotOwner error for update_ownership"
        );

        // Test that owner can transfer ownership
        wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: new_owner.address(),
                expiry: None,
            }),
            &[],
            &admin,
        )
        .unwrap();

        // New owner needs to accept the transfer
        wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership),
            &[],
            &new_owner,
        )
        .unwrap();

        // Verify new owner can execute owner-only functions
        wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateDrawState {
                new_state: DrawState::DrawFinished,
            },
            &[],
            &new_owner,
        )
        .unwrap();

        // Verify old owner cannot execute owner-only functions
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateDrawState {
                new_state: DrawState::TicketSalesOpen,
            },
            &[],
            &admin,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NotOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NotOwner error for old owner"
        );

        // Test that new owner can renounce ownership
        wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateOwnership(cw_ownable::Action::RenounceOwnership),
            &[],
            &new_owner,
        )
        .unwrap();

        // Verify no one can execute owner-only functions after renouncing
        let result = wasm.execute(
            &contract_address,
            &ExecuteMsg::UpdateDrawState {
                new_state: DrawState::TicketSalesOpen,
            },
            &[],
            &new_owner,
        );
        assert!(
            result.unwrap_err().to_string().contains(
                ContractError::Ownership(cw_ownable::OwnershipError::NoOwner)
                    .to_string()
                    .as_str()
            ),
            "Expected NoOwner error after renouncing ownership"
        );
    }
}
