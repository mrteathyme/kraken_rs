use spot::rest::{sign, Payload};

pub mod spot;
pub mod derivatives;
pub mod utils;

pub struct APISignature(Box<str>);
impl APISignature {
    pub fn new(signature: &str) -> Self {
        APISignature(signature.into())
    }
}
impl std::ops::Deref for APISignature {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub struct APISecret<'a>(&'a str);
impl<'a> APISecret<'a> {
    pub const fn new(secret: &'a str) -> Self {
        APISecret(secret)
    }
}

impl std::ops::Deref for APISecret<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub struct APIKey<'a>(&'a str);
impl std::ops::Deref for APIKey<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}
impl<'a> APIKey<'a> {
    pub const fn new(key: &'a str) -> Self {
        APIKey(key)
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum KrakenResponse<T> {
    Spot {
        result: Option<T>, 
        error: Option<serde_json::Value>
    },
    Futures {
        result: String,
        #[serde(rename = "serverTime")]
        server_time: f64,
        #[serde(flatten)]
        data: T,
    },
    FuturesError {
        result: String,
        #[serde(rename = "serverTime")]
        server_time: f64,
        errors: Vec<String>
    }
}

pub struct KrakenRequest<Response: for<'a> serde::Deserialize<'a>>(http::Request<String>,std::marker::PhantomData<Response>);

impl<Response: for<'a> serde::Deserialize<'a> + std::fmt::Debug> KrakenRequest<Response> {
    fn new(request: http::Request<String>) -> Self {
        KrakenRequest(request,std::marker::PhantomData)
    }
    fn new_spot<Params>(
        method: http::Method,
        params: &Params,
        uri: &http::Uri,
        key: &APIKey,
        secret: &APISecret,
    ) -> Self
    where
        Params: serde::Serialize + Payload,
    {
        let path = uri.path_and_query().unwrap();
        Self::new(http::Request::builder()
        .method(method)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("API-Key", key.to_string())
        .header("API-Sign", sign(secret,path.clone(),params).to_string())
        .uri(uri)
        .body(serde_json::to_string(&params).unwrap()).unwrap())
    }

    pub async fn send<F, R, E>(self, func: F) -> Result<Response, Box<dyn std::error::Error>>
    where F: Fn(http::Request<String>) -> R,
        R: std::future::Future<Output = Result<bytes::Bytes, E>>,
        Box<dyn std::error::Error>: From<E>
    {
        let res = func(self.0).await?;
        match serde_json::from_slice(&res) {
            Ok(KrakenResponse::Spot {result, error}) => match result {
                Some(data) => Ok(data),
                None => Err(error.unwrap().to_string().into())
            },
            Ok(KrakenResponse::Futures {data, ..}) => {
                Ok(data)
            }
            Ok(KrakenResponse::FuturesError {errors,..}) => Err(format!("{:?}",errors).into()), //ToDo: Better error handling
            Err(e) => {
                Err(format!("Error parsing response: {}\n Raw: {}", e, String::from_utf8_lossy(&res)).into())
            }
        }
    }
}
