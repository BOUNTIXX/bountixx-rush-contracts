#![no_std]

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, token::TokenClient, Address, BytesN, Env,
};

#[contract]
pub struct RushVault;

#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub struct Config {
    pub admin: Address,
    pub verifier: Address,
}

#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub enum CampaignStatus {
    Funded,
    Settled,
    Refunded,
}

#[contracttype]
#[derive(Clone)]
pub struct Campaign {
    pub sponsor: Address,
    pub verifier: Address,
    pub token: Address,
    pub amount: i128,
    pub manifest_hash: BytesN<32>,
    pub deadline_ledger: u32,
    pub status: CampaignStatus,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    Campaign(BytesN<32>),
}

#[contractevent]
pub struct Funded {
    #[topic]
    pub campaign_id: BytesN<32>,
    pub amount: i128,
}

#[contractevent]
pub struct Settled {
    #[topic]
    pub campaign_id: BytesN<32>,
    pub winner: Address,
    pub result_hash: BytesN<32>,
}

#[contractevent]
pub struct Refunded {
    #[topic]
    pub campaign_id: BytesN<32>,
    pub sponsor: Address,
}

#[contractevent]
pub struct VerifierUpdated {
    #[topic]
    pub admin: Address,
    pub verifier: Address,
}

fn config(env: &Env) -> Config {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_or_else(|| panic!("vault is not initialized"))
}

fn campaign(env: &Env, campaign_id: &BytesN<32>) -> Campaign {
    env.storage()
        .persistent()
        .get(&DataKey::Campaign(campaign_id.clone()))
        .unwrap_or_else(|| panic!("campaign does not exist"))
}

#[contractimpl]
impl RushVault {
    /// Initializes the vault once. The verifier is the authorized off-chain
    /// attester in the first release; it does not hold participant funds.
    pub fn initialize(env: Env, admin: Address, verifier: Address) {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("vault already initialized");
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Config, &Config { admin, verifier });
    }

    /// Rotates the default verifier for campaigns funded after this update.
    /// Existing campaigns retain their verifier so their trust boundary cannot
    /// be changed after a sponsor locks funds and rules.
    pub fn set_verifier(env: Env, verifier: Address) {
        let mut cfg = config(&env);
        cfg.admin.require_auth();
        let admin = cfg.admin.clone();
        cfg.verifier = verifier.clone();
        env.storage().instance().set(&DataKey::Config, &cfg);
        VerifierUpdated { admin, verifier }.publish(&env);
    }

    /// Transfers a sponsor-funded prize into this contract and locks the
    /// campaign manifest before play starts.
    pub fn fund(
        env: Env,
        campaign_id: BytesN<32>,
        sponsor: Address,
        token: Address,
        amount: i128,
        manifest_hash: BytesN<32>,
        deadline_ledger: u32,
    ) {
        let cfg = config(&env);
        sponsor.require_auth();
        if amount <= 0 || deadline_ledger <= env.ledger().sequence() {
            panic!("invalid campaign funding parameters");
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Campaign(campaign_id.clone()))
        {
            panic!("campaign already exists");
        }

        TokenClient::new(&env, &token).transfer(
            &sponsor,
            &env.current_contract_address(),
            &amount,
        );

        let campaign = Campaign {
            sponsor,
            verifier: cfg.verifier,
            token,
            amount,
            manifest_hash,
            deadline_ledger,
            status: CampaignStatus::Funded,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id.clone()), &campaign);
        Funded { campaign_id, amount }.publish(&env);
    }

    /// Settles a funded Rush after the configured verifier has authenticated
    /// the deterministic, off-chain result. `result_hash` is emitted for an
    /// auditable proof trail; it is not an attempt to run arbitrary scoring
    /// logic on-chain.
    pub fn settle(
        env: Env,
        campaign_id: BytesN<32>,
        verifier: Address,
        winner: Address,
        result_hash: BytesN<32>,
    ) {
        verifier.require_auth();
        let mut stored = campaign(&env, &campaign_id);
        if verifier != stored.verifier {
            panic!("unauthorized verifier");
        }

        if stored.status != CampaignStatus::Funded {
            panic!("campaign is not fundable");
        }

        TokenClient::new(&env, &stored.token).transfer(
            &env.current_contract_address(),
            &winner,
            &stored.amount,
        );
        stored.status = CampaignStatus::Settled;
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id.clone()), &stored);
        Settled {
            campaign_id,
            winner,
            result_hash,
        }
        .publish(&env);
    }

    /// Allows a sponsor to recover an unresolved prize only after the locked
    /// deadline. This avoids an organizer withdrawing a live campaign early.
    pub fn refund(env: Env, campaign_id: BytesN<32>, sponsor: Address) {
        sponsor.require_auth();
        let mut stored = campaign(&env, &campaign_id);
        if sponsor != stored.sponsor {
            panic!("only the sponsor can refund");
        }
        if stored.status != CampaignStatus::Funded {
            panic!("campaign cannot be refunded");
        }
        if env.ledger().sequence() < stored.deadline_ledger {
            panic!("campaign deadline has not passed");
        }

        TokenClient::new(&env, &stored.token).transfer(
            &env.current_contract_address(),
            &sponsor,
            &stored.amount,
        );
        stored.status = CampaignStatus::Refunded;
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id.clone()), &stored);
        Refunded {
            campaign_id,
            sponsor,
        }
        .publish(&env);
    }

    pub fn get_campaign(env: Env, campaign_id: BytesN<32>) -> Campaign {
        campaign(&env, &campaign_id)
    }

    /// Exposes the public contract configuration so clients can display the
    /// active default verifier for campaigns that have not been funded yet.
    pub fn get_config(env: Env) -> Config {
        config(&env)
    }
}

mod test;
