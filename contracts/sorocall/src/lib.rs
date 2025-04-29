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
    pub fn init(env: Env, seller: Address, xlm: Address, usdc: Address, oracle: Address) {
        let store = env.storage().persistent();
        store.set(&symbol_short!("seller"), &seller);
        store.set(&symbol_short!("purchased"), &false);
        store.set(&symbol_short!("xlm"), &xlm);
        store.set(&symbol_short!("usdc"), &usdc);
        store.set(&symbol_short!("oracle"), &oracle);
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
        assert!(!purchased, "Option already purchased");
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

    pub fn update_price(env: Env, oracle: Address, price: i128) {
        oracle.require_auth();
        let store = env.storage().persistent();
        let oracle_check: Address = store.get(&symbol_short!("oracle")).unwrap();
        assert!(oracle_check == oracle, "Unauthorized");
        store.set(&symbol_short!("price"), &price);
    }

    pub fn claim(env: Env) {
        let store = env.storage().persistent();
        let purchased: bool = store.get(&symbol_short!("purchased")).unwrap();
        assert!(env.ledger().timestamp() > EXPIRY, "Option not yet expired");
        let price: i128 = store.get(&symbol_short!("price")).unwrap();
        let buyer: Address = store.get(&symbol_short!("buyer")).unwrap();
        let seller: Address = store.get(&symbol_short!("seller")).unwrap();
        let xlm: Address = store.get(&symbol_short!("xlm")).unwrap();
        let usdc: Address = store.get(&symbol_short!("usdc")).unwrap();
        if price < STRIKE || purchased == false {
            TokenClient::new(&env, &xlm).transfer(
                &env.current_contract_address(),
                &seller,
                &AMOUNT_XLM,
            );
        } else {
            let diff = STRIKE - PREMIUM; // 400_000 (0.40 USDC)
            TokenClient::new(&env, &usdc).transfer_from(
                &env.current_contract_address(),
                &buyer,
                &seller,
                &diff,
            );
            TokenClient::new(&env, &xlm).transfer(
                &env.current_contract_address(),
                &buyer,
                &AMOUNT_XLM,
            );
        }
        store.set(&symbol_short!("purchased"), &false);
    }
}

//mod test;
