#[cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _}, testutils::Ledger, Env, Address, String, symbol_short};
use sep_41_token::testutils::{MockTokenClient, MockTokenWASM};

fn create_token_contract(env: &Env) -> (Address, MockTokenClient) {
    let admin = Address::generate(&env);
    let token_id = env.register_contract_wasm(None, MockTokenWASM);
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Name"),
        &String::from_str(&env, "Symbol"),
    );
    (token_id, token_client)
}

fn setup<'a>(env: &'a Env) -> (XlmCallOptionClient<'a>, Address, Address, Address, Address) {
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let loads = &999999999999999_i128;
    let (usdc_id, usdc_token) = create_token_contract(&env);
    let (xlm_id, xlm_token) = create_token_contract(&env);
    let contract_id = env.register_contract(None, XlmCallOption);
    let client = XlmCallOptionClient::new(&env, &contract_id);
    env.mock_all_auths();
    let mock_xlm = MockTokenClient::new(&env, &xlm_id);
    let mock_usdc = MockTokenClient::new(&env, &usdc_id);
    mock_xlm.mint(&seller, &loads);
    xlm_token.approve(&seller, &contract_id, &loads, &0_u32);
    mock_usdc.mint(&buyer, &loads);
    usdc_token.approve(&buyer, &contract_id, &loads, &0_u32);
    client.sell_option(&seller, &xlm_id, &usdc_id);
    (client, seller, buyer, usdc_id, xlm_id)
}

#[test]
fn test_init() {
    let env = Env::default();
    let (client, seller, _buyer, usdc_id, xlm_id) = setup(&env);
    env.as_contract(&client.address, || {
        let stored_seller: Address = env.storage().persistent().get(&symbol_short!("seller")).unwrap();
        let stored_usdc_id: Address = env.storage().persistent().get(&symbol_short!("usdc")).unwrap();
        let stored_xlm_id: Address = env.storage().persistent().get(&symbol_short!("xlm")).unwrap();
        assert_eq!(stored_seller, seller);
        assert_eq!(stored_usdc_id, usdc_id);
        assert_eq!(stored_xlm_id, xlm_id);
    });
}

#[test]
fn test_purchase_option() {
    let env = Env::default();
    let (client, _seller, buyer, _usdc_id, _xlm_id) = setup(&env);
    client.purchase_option(&buyer);
    env.as_contract(&client.address, || {
        let purchased: bool = env.storage().persistent().get(&symbol_short!("purchased")).unwrap();
        assert_eq!(purchased, true);
    });
}

#[test]
#[should_panic(expected = "Option already purchased")]
fn test_double_purchase() {
    let env = Env::default();
    let (client, _seller, buyer, _usdc_id, _xlm_id) = setup(&env);
    client.purchase_option(&buyer);
    client.purchase_option(&buyer);
}

#[test]
fn test_exercise() {
    let env = Env::default();
    let (client, _seller, buyer, _usdc_id, _xlm_id) = setup(&env);
    client.purchase_option(&buyer);
    env.ledger().with_mut(|li| {
        li.timestamp = EXPIRY - 1;
    });
    client.exercise(&buyer);
    env.as_contract(&client.address, || {
        let purchased: bool = env.storage().persistent().get(&symbol_short!("purchased")).unwrap();
        assert_eq!(purchased, false);
    });
}

#[test]
#[should_panic(expected = "Option already exercised")]
fn test_double_exercise() {
    let env = Env::default();
    let (client, _seller, buyer, _usdc_id, _xlm_id) = setup(&env);
    client.purchase_option(&buyer);
    client.exercise(&buyer);
    client.exercise(&buyer);
}

#[test]
fn test_expire() {
    let env = Env::default();
    let (client, _seller, buyer, _usdc_id, _xlm_id) = setup(&env);
    client.purchase_option(&buyer);
    env.ledger().with_mut(|li| {
        li.timestamp = EXPIRY + 1;
    });
    client.expire();
    env.as_contract(&client.address, || {
        let purchased: bool = env.storage().persistent().get(&symbol_short!("purchased")).unwrap();
        assert_eq!(purchased, false);
    });
}
