#[cfg(test)]
mod tests {
    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        state::{Config, DrawState},
    };
    use coreum_test_tube::{
        Account, AssetFT, Bank, CoreumTestApp, Module, SigningAccount, Staking, Wasm,
    };
    use coreum_wasm_sdk::types::cosmos::bank::v1beta1::MsgSend;
    use coreum_wasm_sdk::types::cosmos::staking::v1beta1::{
        CommissionRates, Description, MsgCreateValidator,
    };

    use bech32::{Bech32, Hrp};
    use coreum_wasm_sdk::types::coreum::asset::ft::v1::{
        MsgBurn, MsgIssue, MsgMint, QueryBalanceRequest, QueryBalanceResponse,
    };

    use coreum_wasm_sdk::shim::Any;
    use coreum_wasm_sdk::types::cosmos::base::v1beta1::Coin as BaseCoin;
    use cosmrs::crypto::secp256k1::SigningKey;
    use cosmrs::proto;
    use cosmrs::proto::cosmos::crypto::ed25519::PubKey;
    use cosmrs::AccountId;
    use cosmwasm_std::{coin, coins, Addr, Coin as CosmoCoin, CosmosMsg, StakingMsg, Uint128};
    use prost::Message;
    use ring::{
        rand,
        signature::{self, KeyPair},
    };
    use std::str::FromStr;

    const FEE_DENOM: &str = "ucore";
    const TICKET_TOKEN: &str = "TICKET";
    const TICKET_PRECISION: u32 = 6;
    const SECONDS_PER_BLOCK: u64 = 11; // 1.1 seconds = 11/10 seconds
    const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
    const UNDELEGATION_DAYS: u64 = 7;
    const TICKET_PRICE: u128 = 200_000_000; //200 COREUM

    // fn derive_validator_address(account_address: &str) -> String {
    //     // Parse the account address to get the public key bytes
    //     let account_id = AccountId::from_str(account_address).unwrap();
    //     let pubkey_bytes = account_id.as_ref();

    //     // Encode with the validator prefix
    //     let validator_addr =
    //         encode("corevaloper", pubkey_bytes.to_base32(), Variant::Bech32).unwrap();
    //     validator_addr
    // }

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
        let staking: Staking<'_, CoreumTestApp> = Staking::new(&app);

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
        let user = app
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
        let payment = number_of_tickets * Uint128::from(TICKET_PRICE);

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

        let undelegation_period_seconds: u64 = SECONDS_PER_DAY * 1000;
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
            amount: number_of_tickets.pow(TICKET_PRECISION),
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
}
