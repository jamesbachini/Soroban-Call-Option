#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Address};
use sep_41_token::TokenClient;

const AMOUNT_XLM: i128 = 1;      // 1 XLM collateral
const STRIKE: i128 = 500_000;    // 0.50 USDC (6 decimals)
const PREMIUM: i128 = 100_000;   // 0.10 USDC
const EXPIRY: u64 = 1767225600;  // Jan 1, 2026 UTC

#[contract]
pub struct XlmCallOption;

#[contractimpl]
impl XlmCallOption {
    pub fn sell_option(env: Env, seller: Address, xlm: Address, usdc: Address) {
        let store = env.storage().persistent();
        if store.get::<_, Address>(&symbol_short!("seller")).is_some() {
            panic!("Option already sold");
        }
        store.set(&symbol_short!("seller"), &seller);
        store.set(&symbol_short!("purchased"), &false);
        store.set(&symbol_short!("xlm"), &xlm);
        store.set(&symbol_short!("usdc"), &usdc);
        TokenClient::new(&env, &xlm).transfer_from(
            &env.current_contract_address(),
            &seller,
            &env.current_contract_address(),
            &AMOUNT_XLM,
        );
    }

    pub fn purchase_option(env: Env, buyer: Address) {
        let store = env.storage().persistent();
        let purchased: bool = store.get(&symbol_short!("purchased")).unwrap();
        assert!(purchased == false, "Option already purchased");
        assert!(env.ledger().timestamp() < EXPIRY, "Option expired");
        let seller: Address = store.get(&symbol_short!("seller")).unwrap();
        let usdc: Address = store.get(&symbol_short!("usdc")).unwrap();
        TokenClient::new(&env, &usdc).transfer_from(
            &env.current_contract_address(),
            &buyer,
            &seller,
            &PREMIUM,
        );
        store.set(&symbol_short!("buyer"), &buyer);
        store.set(&symbol_short!("purchased"), &true);
    }

    pub fn exercise(env: Env, exerciser: Address) {
        let store = env.storage().persistent();
        exerciser.require_auth();
        assert!(env.ledger().timestamp() < EXPIRY, "Option has expired");
        let purchased: bool = store.get(&symbol_short!("purchased")).unwrap();
        assert!(purchased == true, "Option already exercised");
        let buyer: Address = store.get(&symbol_short!("buyer")).unwrap();
        assert!(buyer == exerciser, "Not your option");
        let seller: Address = store.get(&symbol_short!("seller")).unwrap();
        let xlm: Address = store.get(&symbol_short!("xlm")).unwrap();
        let usdc: Address = store.get(&symbol_short!("usdc")).unwrap();
        TokenClient::new(&env, &usdc).transfer_from(
            &env.current_contract_address(),
            &buyer,
            &seller,
            &STRIKE,
        );
        TokenClient::new(&env, &xlm).transfer(
            &env.current_contract_address(),
            &buyer,
            &AMOUNT_XLM,
        );
        store.set(&symbol_short!("purchased"), &false);
    }

    pub fn expire(env: Env) {
        let store = env.storage().persistent();
        assert!(env.ledger().timestamp() > EXPIRY, "Option not yet expired");
        let seller: Address = store.get(&symbol_short!("seller")).unwrap();
        let xlm: Address = store.get(&symbol_short!("xlm")).unwrap();
        TokenClient::new(&env, &xlm).transfer(
            &env.current_contract_address(),
            &seller,
            &AMOUNT_XLM,
        );
        store.set(&symbol_short!("purchased"), &false);
    }
}

mod test;
