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
    pub fn new(secret: &'a str) -> Self {
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
    pub fn new(key: &'a str) -> Self {
        APIKey(key)
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum KrakenResponse<T> {
    Spot {
        response: Option<T>, 
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

pub struct Request<Response: for<'a> serde::Deserialize<'a>>(http::Request<String>,std::marker::PhantomData<Response>);

impl<Response: for<'a> serde::Deserialize<'a>> Request<Response> {
    fn new(request: http::Request<String>) -> Self {
        Request(request,std::marker::PhantomData)
    }
    pub async fn send<F, R, E>(self, func: F) -> Result<Response, Box<dyn std::error::Error>>
    where F: Fn(http::Request<String>) -> R,
        R: std::future::Future<Output = Result<bytes::Bytes, E>>,
        Box<dyn std::error::Error>: From<E>
    {
        let response: KrakenResponse<Response> = serde_json::from_slice(&func(self.0).await?)?;
        match response {
            KrakenResponse::Spot {response, error} => match response {
                Some(data) => Ok(data),
                None => Err(error.unwrap().to_string().into())
            },
            KrakenResponse::Futures {data, ..} => {
                Ok(data)
            }
            KrakenResponse::FuturesError {errors,..} => Err(format!("{:?}",errors).into()) //ToDo: Better error handling
        }
    }
}
