use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use crate::{spot::rest::Payload, APIKey, APISecret, KrakenRequest};

#[derive(Deserialize, Clone, Debug, Copy)]
pub struct Balance {
    spot: f64,
    earn: f64,
    staked: f64,
    opt_in_rewards: f64,
    yield_bearing: f64,
}

impl Balance {
    pub const EMPTY: Self = Self {
        spot: 0.0,
        earn: 0.0,
        staked: 0.0,
        opt_in_rewards: 0.0,
        yield_bearing: 0.0,
    };

    pub fn total(&self) -> f64 {
        self.spot + self.earn + self.staked + self.opt_in_rewards + self.yield_bearing
    }
    pub fn available(&self) -> f64 {
        self.spot + self.earn
    }
}

#[derive(Clone, Debug)]
pub struct AccountBalance(HashMap<String, Balance>);
impl Deref for AccountBalance {
    type Target = HashMap<String, Balance>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for AccountBalance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = HashMap::<String, String>::deserialize(deserializer)?;
        let mut balance_map = HashMap::new();
        for (key, value) in map.iter() {
            if key.contains(".") {
                let parts: Vec<&str> = key.split('.').collect();
                let balance = balance_map.entry(parts[0].to_string()).or_insert(Balance::EMPTY);
                match parts[1] {
                    "F" => balance.earn = value.parse().unwrap_or(0.0),
                    "S" => balance.staked = value.parse().unwrap_or(0.0),
                    "M" => balance.opt_in_rewards = value.parse().unwrap_or(0.0),
                    "B" => balance.yield_bearing = value.parse().unwrap_or(0.0),
                    _ => {}
                }
            } else {
                let balance = balance_map.entry(key.to_string()).or_insert(Balance {
                    spot: 0.0,
                    earn: 0.0,
                    staked: 0.0,
                    opt_in_rewards: 0.0,
                    yield_bearing: 0.0,
                });
                balance.spot = value.parse().unwrap_or(0.0);
            }
        }
        Ok(AccountBalance(balance_map))
    }
}

pub fn get_account_balance(key: &APIKey, secret: &APISecret, nonce: i64) -> KrakenRequest<AccountBalance> { //Todo: model asset and asset_class as types
    #[derive(Serialize)]
    struct Parameters {
        nonce: i64,
    }
    impl Payload for Parameters {
        fn nonce(&self) -> i64 {
            self.nonce
        }
    }
    let params = Parameters {
        nonce,
    };
    let uri = http::uri::Uri::from_static("https://api.kraken.com/0/private/Balance");
    KrakenRequest::new_spot(http::Method::POST,&params,&uri,key,&secret)
}
