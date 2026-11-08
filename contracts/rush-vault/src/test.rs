#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, BytesN, Env,
};

const PRIZE: i128 = 500_000_000;

fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn set_ledger_sequence(env: &Env, sequence_number: u32) {
    env.ledger()
        .with_mut(|ledger| ledger.sequence_number = sequence_number);
}

fn assert_last_auth_is(env: &Env, expected: &Address) {
    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, expected.clone());
}

struct Fixture {
    env: Env,
    vault_id: Address,
    token_id: Address,
    admin: Address,
    sponsor: Address,
    verifier: Address,
    winner: Address,
}

impl Fixture {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let sponsor = Address::generate(&env);
        let verifier = Address::generate(&env);
        let winner = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(token_admin).address();
        let vault_id = env.register(RushVault, ());

        let vault = RushVaultClient::new(&env, &vault_id);
        let asset = token::StellarAssetClient::new(&env, &token_id);
        vault.initialize(&admin, &verifier);
        asset.mint(&sponsor, &PRIZE);

        Self {
            env,
            vault_id,
            token_id,
            admin,
            sponsor,
            verifier,
            winner,
        }
    }

    fn vault(&self) -> RushVaultClient<'_> {
        RushVaultClient::new(&self.env, &self.vault_id)
    }

    fn token(&self) -> token::TokenClient<'_> {
        token::TokenClient::new(&self.env, &self.token_id)
    }

    fn asset(&self) -> token::StellarAssetClient<'_> {
        token::StellarAssetClient::new(&self.env, &self.token_id)
    }

    fn fund(&self, campaign_id: &BytesN<32>, deadline_ledger: u32) {
        self.vault().fund(
            campaign_id,
            &self.sponsor,
            &self.token_id,
            &PRIZE,
            &hash(&self.env, 1),
            &deadline_ledger,
        );
        assert_last_auth_is(&self.env, &self.sponsor);
    }
}

#[test]
fn initializes_once() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(RushVault, ());
    let client = RushVaultClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let verifier = Address::generate(&env);

    client.initialize(&admin, &verifier);
    assert_last_auth_is(&env, &admin);
    assert!(client.try_initialize(&admin, &verifier).is_err());
}

#[test]
fn sponsor_funds_and_authorized_verifier_settles_once() {
    let fixture = Fixture::new();
    let campaign_id = hash(&fixture.env, 10);
    let result_hash = hash(&fixture.env, 11);

    fixture.fund(&campaign_id, 100);
    assert_eq!(fixture.token().balance(&fixture.sponsor), 0);
    assert_eq!(fixture.token().balance(&fixture.vault_id), PRIZE);

    let funded = fixture.vault().get_campaign(&campaign_id);
    assert_eq!(funded.sponsor, fixture.sponsor);
    assert_eq!(funded.verifier, fixture.verifier);
    assert!(funded.status == CampaignStatus::Funded);

    fixture
        .vault()
        .settle(&campaign_id, &fixture.verifier, &fixture.winner, &result_hash);
    assert_last_auth_is(&fixture.env, &fixture.verifier);

    assert_eq!(fixture.token().balance(&fixture.vault_id), 0);
    assert_eq!(fixture.token().balance(&fixture.winner), PRIZE);
    assert!(fixture.vault().get_campaign(&campaign_id).status == CampaignStatus::Settled);

    assert!(fixture
        .vault()
        .try_settle(&campaign_id, &fixture.verifier, &fixture.winner, &result_hash)
        .is_err());
    assert_eq!(fixture.token().balance(&fixture.winner), PRIZE);
}

#[test]
fn sponsor_can_refund_only_after_deadline_and_only_once() {
    let fixture = Fixture::new();
    let campaign_id = hash(&fixture.env, 20);
    fixture.fund(&campaign_id, 100);

    assert!(fixture
        .vault()
        .try_refund(&campaign_id, &fixture.sponsor)
        .is_err());
    assert_eq!(fixture.token().balance(&fixture.vault_id), PRIZE);

    set_ledger_sequence(&fixture.env, 101);
    fixture.vault().refund(&campaign_id, &fixture.sponsor);
    assert_last_auth_is(&fixture.env, &fixture.sponsor);

    assert_eq!(fixture.token().balance(&fixture.vault_id), 0);
    assert_eq!(fixture.token().balance(&fixture.sponsor), PRIZE);
    assert!(fixture.vault().get_campaign(&campaign_id).status == CampaignStatus::Refunded);

    assert!(fixture
        .vault()
        .try_refund(&campaign_id, &fixture.sponsor)
        .is_err());
    assert!(fixture
        .vault()
        .try_settle(
            &campaign_id,
            &fixture.verifier,
            &fixture.winner,
            &hash(&fixture.env, 21),
        )
        .is_err());
    assert_eq!(fixture.token().balance(&fixture.sponsor), PRIZE);
}

#[test]
fn funded_campaigns_keep_their_original_verifier_after_rotation() {
    let fixture = Fixture::new();
    let first_campaign = hash(&fixture.env, 30);
    fixture.fund(&first_campaign, 100);

    let next_verifier = Address::generate(&fixture.env);
    fixture.vault().set_verifier(&next_verifier);
    assert_last_auth_is(&fixture.env, &fixture.admin);
    assert_eq!(fixture.vault().get_config().verifier, next_verifier);
    assert_eq!(
        fixture.vault().get_campaign(&first_campaign).verifier,
        fixture.verifier
    );

    assert!(fixture
        .vault()
        .try_settle(
            &first_campaign,
            &next_verifier,
            &fixture.winner,
            &hash(&fixture.env, 31),
        )
        .is_err());
    fixture.vault().settle(
        &first_campaign,
        &fixture.verifier,
        &fixture.winner,
        &hash(&fixture.env, 31),
    );
    assert_last_auth_is(&fixture.env, &fixture.verifier);

    fixture.asset().mint(&fixture.sponsor, &PRIZE);
    let future_campaign = hash(&fixture.env, 32);
    fixture.fund(&future_campaign, 100);
    assert_eq!(
        fixture.vault().get_campaign(&future_campaign).verifier,
        next_verifier
    );
}

#[test]
fn rejects_an_unknown_campaign() {
    let env = Env::default();
    let contract_id = env.register(RushVault, ());
    let client = RushVaultClient::new(&env, &contract_id);
    let missing = BytesN::from_array(&env, &[0; 32]);
    assert!(client.try_get_campaign(&missing).is_err());
}
