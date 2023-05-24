use anyhow::{
    anyhow,
    Result,
};
use assert2::assert;

use aleph_client::Balance;
use ink_wrapper_types::{
    util::ToAccountId,
    Connection,
};

use crate::{
    events::{
        get_burn_events,
        get_mint_events,
        get_pair_created_events,
        get_swap_events,
    },
    factory_contract,
    factory_contract::Factory,
    pair_contract,
    pair_contract::{
        Pair,
        PSP22 as PairPSP22,
    },
    psp22_token,
    psp22_token::PSP22 as TokenPSP22,
    test::setup::{
        Contracts,
        TestFixture,
        EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
        ZERO_ADDRESS,
    },
};

const EXPECTED_ALL_PAIR_LENGTH: u64 = 1;
const BALANCE: Balance = 10_000;
const MIN_BALANCE: Balance = 1_000;
const EXPECTED_INITIAL_NON_SUDO_BALANCE: Balance = 0;

const FIRST_AMOUNT_IN: Balance = 1_020;
const FIRST_AMOUNT_OUT: Balance = 0;
const SECOND_AMOUNT_OUT: Balance = 900;

const FIRST_BALANCE_LOCKED: Balance = 2_204;
const SECOND_BALANCE_LOCKED: Balance = 1_820;
const PAIR_TRANSFER: Balance = 2_000;

pub async fn create_pair(test_fixture: &TestFixture) -> Result<()> {
    let TestFixture {
        sudo_connection,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a: first_token,
        token_b: second_token,
        ..
    } = contracts;

    let all_pairs_length_before = factory_contract.all_pairs_length(sudo_connection).await??;

    assert!(all_pairs_length_before == EXPECTED_INITIAL_ALL_PAIRS_LENGTH);

    let token_a: ink_primitives::AccountId = (*first_token).into();
    let token_b: ink_primitives::AccountId = (*second_token).into();

    let tx_info = factory_contract
        .create_pair(sudo_connection, token_a, token_b)
        .await?;

    let all_events = sudo_connection.get_contract_events(tx_info).await?;
    let contract_events = all_events.for_contract(*factory_contract);
    let pair_created_events = get_pair_created_events(contract_events);
    let first_pair_created_event = pair_created_events
        .first()
        .ok_or(anyhow!("No `PairCreated` events have been emitted!"))?;
    let factory_contract::event::Event::PairCreated {
        token_0,
        token_1,
        pair,
        pair_len,
    } = first_pair_created_event;

    let mut expected_token_pair = vec![token_a, token_b];
    expected_token_pair.sort();
    let actual_token_pair = vec![*token_0, *token_1];

    assert!(*pair != ZERO_ADDRESS.into());
    assert!(actual_token_pair == expected_token_pair);
    assert!(*pair_len == EXPECTED_ALL_PAIR_LENGTH);

    let all_pairs_length_after = factory_contract.all_pairs_length(sudo_connection).await??;

    assert!(all_pairs_length_after == EXPECTED_ALL_PAIR_LENGTH);

    Ok(())
}

pub async fn mint_pair(test_fixture: &TestFixture) -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    let pair = factory_contract
        .get_pair(sudo_connection, (*token_a).into(), (*token_b).into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    token_a
        .transfer(sudo_connection, pair, BALANCE, vec![])
        .await?;
    token_b
        .transfer(sudo_connection, pair, BALANCE, vec![])
        .await?;

    let pair_contract: pair_contract::Instance = pair.into();
    let non_sudo_ink_account = non_sudo.account_id().to_account_id();
    let non_sudo_balance_before = pair_contract
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_before == EXPECTED_INITIAL_NON_SUDO_BALANCE);

    let mint_tx_info = pair_contract
        .mint(sudo_connection, non_sudo_ink_account)
        .await?;

    let all_pair_contract_events = sudo_connection.get_contract_events(mint_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let mint_events = get_mint_events(pair_contract_events);
    mint_events
        .first()
        .ok_or(anyhow!("No `Mint` events have been emitted!"))?;

    let expected_balance = BALANCE - MIN_BALANCE;
    let non_sudo_balance_after = pair_contract
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_after == expected_balance);

    Ok(())
}

pub async fn swap_tokens(test_fixture: &TestFixture) -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    let pair = factory_contract
        .get_pair(sudo_connection, (*token_a).into(), (*token_b).into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    let (first_token, second_token) = sort_tokens(*token_a, *token_b);
    first_token
        .transfer(sudo_connection, pair, FIRST_AMOUNT_IN, vec![])
        .await?;
    let non_sudo_ink_account = non_sudo.account_id().to_account_id();
    let non_sudo_balance_before = second_token
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_before == EXPECTED_INITIAL_NON_SUDO_BALANCE);

    let pair_contract: pair_contract::Instance = pair.into();
    let swap_tx_info = pair_contract
        .swap(
            sudo_connection,
            FIRST_AMOUNT_OUT,
            SECOND_AMOUNT_OUT,
            non_sudo_ink_account,
        )
        .await?;

    let all_pair_contract_events = sudo_connection.get_contract_events(swap_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let swap_events = get_swap_events(pair_contract_events);
    swap_events
        .first()
        .ok_or(anyhow!("No `Swap` events have been emitted!"))?;

    let non_sudo_balance_after = second_token
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_after == SECOND_AMOUNT_OUT);

    Ok(())
}

pub async fn burn_liquidity_provider_token(test_fixture: &TestFixture) -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    let (first_token, second_token) = sort_tokens(*token_a, *token_b);
    let non_sudo_ink_account = non_sudo.account_id().to_account_id();
    let first_token_balance_before = first_token
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;
    let second_token_balance_before = second_token
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;
    let pair = factory_contract
        .get_pair(sudo_connection, first_token.into(), second_token.into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;

    let pair_contract: pair_contract::Instance = pair.into();
    pair_contract
        .transfer(non_sudo_connection, pair, PAIR_TRANSFER, vec![])
        .await?;
    let burn_tx_info = pair_contract
        .burn(non_sudo_connection, non_sudo_ink_account)
        .await?;

    let all_pair_contract_events = sudo_connection.get_contract_events(burn_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let burn_events = get_burn_events(pair_contract_events);
    burn_events
        .first()
        .ok_or(anyhow!("No `Burn` events have been emitted!"))?;

    let first_token_balance_after = first_token
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;
    let second_token_balance_after = second_token
        .balance_of(sudo_connection, non_sudo_ink_account)
        .await??;
    let first_token_balance_diff = first_token_balance_after - first_token_balance_before;
    let second_token_balance_diff = second_token_balance_after - second_token_balance_before;

    assert!(first_token_balance_diff == FIRST_BALANCE_LOCKED);
    assert!(second_token_balance_diff == SECOND_BALANCE_LOCKED);

    Ok(())
}

pub fn sort_tokens(
    token_a: psp22_token::Instance,
    token_b: psp22_token::Instance,
) -> (psp22_token::Instance, psp22_token::Instance) {
    let mut tokens: Vec<ink_primitives::AccountId> = vec![token_a.into(), token_b.into()];
    tokens.sort();

    (tokens[0].into(), tokens[1].into())
}
