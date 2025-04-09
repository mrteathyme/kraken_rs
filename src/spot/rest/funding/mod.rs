use std::collections::HashMap;
use std::ops::Deref;

use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::{KrakenRequest, APIKey, APISecret};
use crate::spot::rest::Payload;

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum BoolUnion<T> {
    Bool(bool),
    Data(T)
}

#[derive(serde::Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ReferenceID(pub String);

impl Deref for ReferenceID {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[serde_as]
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Withdrawal {
    pub method: String,
    pub asset: String,
    pub network: String,
    pub aclass: String,
    pub refid: ReferenceID,
    pub txid: String,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub fee: f64,
    pub time: u64,
    pub status: String,
    pub key: String,
}


#[derive(Clone, Debug)]
pub struct Withdrawals(pub HashMap<ReferenceID, Withdrawal>);
impl Deref for Withdrawals {
    type Target = HashMap<ReferenceID, Withdrawal>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for Withdrawals {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = Vec::<Withdrawal>::deserialize(deserializer)?;
        let mut withdrawals_map = HashMap::new();
        for withdrawal in data {
            let refid = withdrawal.refid.clone();
            withdrawals_map.insert(refid, withdrawal);
        }
        Ok(Withdrawals(withdrawals_map))
    }
}

pub fn withdraw_status(key: &APIKey, secret: &APISecret, nonce: i64, asset: Option<&str>, aclass: Option<&str>, method: Option<&str>, start: Option<u64>, end: Option<u64>) -> KrakenRequest<Withdrawals> { //Todo: model asset and asset_class as types
    #[serde_as]
    #[derive(serde::Serialize)]
    struct Parameters<'a> {
        nonce: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        asset: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aclass: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        method: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<DisplayFromStr>")]
        start: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<DisplayFromStr>")]
        end: Option<u64>,
    }
    impl<'a> Payload for Parameters<'a> {
        fn nonce(&self) -> i64 {
            self.nonce
        }
    }
    let params = Parameters {
        nonce,
        aclass,
        asset,
        method,
        start,
        end
    };
    let uri = http::uri::Uri::from_static("https://api.kraken.com/0/private/WithdrawStatus");
    KrakenRequest::new_spot(http::Method::POST,&params,&uri,key,&secret)
}





//ToDo: Move this to lib for easier importing (or maybe a prelude or reexport?)
//ToDo: Implement sanity check by changing address to a network enum that checks for malformed
//address on creation
//checking if the asset supports the network would also probably be nice but given the number of
//assets available might be better to do that at runtime (I believe you can get that information
//from the api)
#[derive(Serialize, Clone, Debug)]
pub struct KrakenWithdrawalAddress<'a> {
    pub address: &'a str,
    pub key: &'a str,
    pub asset: &'a str
}

impl<'a> KrakenWithdrawalAddress<'a> {
    pub fn new(asset: &'a str, address: &'a str, key: &'a str) -> Self {
        KrakenWithdrawalAddress {asset, address, key }
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct WithdrawResponse {
    pub refid: ReferenceID,
}

pub fn withdraw(key: &APIKey, secret: &APISecret, nonce: i64, address: KrakenWithdrawalAddress, amount: f64, max_fee: Option<f64>) -> KrakenRequest<WithdrawResponse> { //Todo: model asset and asset_class as types
    #[serde_as]
    #[derive(serde::Serialize)]
    struct Parameters<'a> {
        nonce: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<DisplayFromStr>")]
        max_fee: Option<f64>,
        #[serde(flatten)]
        address: KrakenWithdrawalAddress<'a>,
        #[serde_as(as = "DisplayFromStr")]
        amount: f64,
    }
    impl<'a> Payload for Parameters<'a> {
        fn nonce(&self) -> i64 {
            self.nonce
        }
    }
    let params = Parameters {
        nonce,
        address,
        amount,
        max_fee
    };
    let uri = http::uri::Uri::from_static("https://api.kraken.com/0/private/Withdraw");
    KrakenRequest::new_spot(http::Method::POST,&params,&uri,key,&secret)
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DepositMethod {
    pub method: Option<String>,
    pub limit: Option<BoolUnion<u64>>, //is either a number or false so we need this wrapper type or a custom deserializer and honestly im too lazy for that 
    #[serde(rename = "address-setup-fee")]
    pub address_setup_fee: Option<String>,
    pub fee: Option<String>,
    #[serde(rename = "gen-address")]
    pub gen_address: Option<bool>,
    pub minimum: Option<String>
}

pub fn deposit_methods(key: &APIKey, secret: &APISecret, nonce: i64, asset: &str, asset_class: Option<&str>) -> KrakenRequest<Vec<DepositMethod>> { //Todo: model asset and asset_class as types
    #[derive(serde::Serialize)]
    struct Parameters<'a> {
        nonce: i64,
        asset: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        aclass: Option<&'a str>
    }
    impl<'a> Payload for Parameters<'a> {
        fn nonce(&self) -> i64 {
            self.nonce
        }
    }
    let params = Parameters {
        nonce,
        asset,
        aclass: asset_class
    };
    let uri = http::uri::Uri::from_static("https://api.kraken.com/0/private/DepositMethods");
    KrakenRequest::new_spot(http::Method::POST,&params,&uri,key,&secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_methods() {
        let key = APIKey::new("ZmFrZV9hcGlfa2V5");
        let secret = APISecret::new("ZmFrZV9zZWNyZXRfa2V5");
        let nonce = 0;
        let asset = "BTC";
        let asset_class = Some("currency");
        let request = deposit_methods(&key,&secret,nonce,asset,asset_class);
        assert_eq!(request.0.uri(),"https://api.kraken.com/0/private/DepositMethods");
        assert_eq!(request.0.headers().get("API-Key").unwrap(),"ZmFrZV9hcGlfa2V5");
        assert_eq!(request.0.headers().get("API-Sign").unwrap(),"KvAplEL4y7lgJNE1yITu4iI9lmS+EG5oJbpHfTUjVFrhDWYBzE4GCR3BQPfFHd1ai3R5PC+vs/+zGy2ennfqmQ==");
        assert_eq!(request.0.body(),"{\"nonce\":0,\"asset\":\"BTC\",\"aclass\":\"currency\"}");
    }
}
