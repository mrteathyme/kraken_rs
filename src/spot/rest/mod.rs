pub mod funding;

use std::ops::Deref;

use crate::{APISecret, APISignature};
use serde::Serialize;


pub trait Payload: Serialize {
    fn nonce(&self) -> i64;
}

pub fn sign<P: Payload>(secret: &APISecret, method: http::uri::PathAndQuery, payload: &P) -> APISignature { //ToDo: this only supports spot auth, refactor later
    let encoded = format!("{}{}",payload.nonce(),serde_json::to_string(&payload).unwrap());
    let digest = ring::digest::digest(&ring::digest::SHA256,encoded.as_bytes());
    let message = [method.path().as_bytes(), digest.as_ref()].concat();
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA512, &base64::decode(secret.deref()).unwrap());//ToDo: Find new base64 crate, because fuck that new api lol
    APISignature::new(&base64::encode(ring::hmac::sign(&key, &message)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sign() {
        let secret = APISecret::new("kQH5HW/8p1uGOVjbgWA7FunAmGO8lsSUXNsu3eow76sz84Q18fWxnyRzBHCd3pd5nE9qa99HAZtuZuj6F1huXg==");
        let nonce = 1616492376594;
        #[derive(serde::Serialize)]
        struct Payload<'a> {
            nonce: i64,
            ordertype: &'a str,
            pair: &'a str,
            price: &'a str,
            #[serde(rename = "type")]
            type_: &'a str,
            volume: &'a str
        }
        impl<'a> super::Payload for Payload<'a> {
            fn nonce(&self) -> i64 {
                self.nonce
            }
        }
        let params = Payload {
            nonce,
            ordertype: "limit",
            pair: "XBTUSD",
            price: "37500",
            type_: "buy",
            volume: "1.25"
        };
        let method = http::uri::PathAndQuery::from_static("/0/private/AddOrder");
        let signature = sign(&secret,method,&params);
        assert_eq!(*signature,*"kMkTQfyYJH05IdnWQ9TIqL9Kq+dKqcD5O/TGPPLRwwy1is/YvqEYtMAHf7tXsqwfbLwp7pbzJzWHxzKPnL8rfA=="); //testcase differs from documentation due to using json rather than urlencoded post data
    }
}
