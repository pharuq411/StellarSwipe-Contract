#![cfg(test)]

extern crate std;

use super::*;
use crate::errors::AdminError;
use soroban_sdk::{testutils::Address as _, Env};

fn setup() -> (Env, SignalRegistryClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    #[allow(deprecated)]
    let contract_id = env.register_contract(None, SignalRegistry);
    let client = SignalRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, client, admin)
}

#[test]
fn two_step_admin_transfer_flow() {
    let (env, client, admin1) = setup();
    let admin2 = Address::generate(&env);

    client.propose_admin_transfer(&admin1, &admin2);
    client.pause_trading(&admin1);
    assert!(client.is_paused());
    client.unpause_trading(&admin1);

    client.accept_admin_transfer(&admin2);
    assert_eq!(client.get_admin(), admin2);

    assert!(client.try_pause_trading(&admin1).is_err());
    client.pause_trading(&admin2);
    assert!(client.is_paused());
}

#[test]
fn admin_transfer_expires() {
    let (env, client, admin) = setup();
    let pending_admin = Address::generate(&env);

    client.propose_admin_transfer(&admin, &pending_admin);

    use soroban_sdk::testutils::Ledger;
    env.ledger()
        .with_mut(|ledger| ledger.sequence_number += 34_560 + 1);

    let result = client.try_accept_admin_transfer(&pending_admin);
    assert_eq!(result, Err(Ok(AdminError::AdminTransferExpired)));
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn admin_transfer_can_be_cancelled() {
    let (env, client, admin) = setup();
    let pending_admin = Address::generate(&env);

    client.propose_admin_transfer(&admin, &pending_admin);
    client.cancel_admin_transfer(&admin);

    let result = client.try_accept_admin_transfer(&pending_admin);
    assert_eq!(result, Err(Ok(AdminError::NoPendingAdminTransfer)));
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn only_pending_admin_can_accept_transfer() {
    let (env, client, admin) = setup();
    let pending_admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.propose_admin_transfer(&admin, &pending_admin);

    let result = client.try_accept_admin_transfer(&attacker);
    assert_eq!(result, Err(Ok(AdminError::Unauthorized)));
    assert_eq!(client.get_admin(), admin);
}
