use crate::{KrakenRequest, APIKey, APISecret};
use crate::spot::rest::{Payload, sign};

#[derive(serde::Deserialize, Clone)]
#[serde(untagged)]
pub enum BoolUnion<T> {
    Bool(bool),
    Data(T)
}

#[derive(serde::Deserialize, Clone)]
pub struct DepositMethod {
    pub method: String,
    pub limit: BoolUnion<u64>, //is either a number or false so we need this wrapper type or a custom deserializer and honestly im too lazy for that 
    #[serde(rename = "address-setup-fee")]
    pub address_setup_fee: String,
    pub fee: String,
    #[serde(rename = "gen-address")]
    pub gen_address: String,
    pub minimum: String
}

pub fn deposit_methods(key: &APIKey, secret: &APISecret, nonce: i64, asset: &str, asset_class: Option<&str>) -> KrakenRequest<Vec<DepositMethod>> { //Todo: model asset and asset_class as types
    #[derive(serde::Serialize)]
    struct Parameters<'a> {
        nonce: i64,
        asset: &'a str,
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
    let path = uri.path_and_query().unwrap();
    let request = http::request::Builder::new()
            .method("POST")
            .header("API-Key", key.to_string())
            .header("API-Sign", sign(secret,path.clone(),&params).to_string())
            .uri(uri)
            .body(serde_json::to_string(&params).unwrap()).unwrap();
    KrakenRequest::new(request)
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
