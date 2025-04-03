pub mod rest;
pub mod websocket;

/*
pub struct SpotRequest<Response: for<'a> serde::Deserialize<'a>>(http::Request<String>,std::marker::PhantomData<Response>);

#[derive(serde::Deserialize)]
pub struct SpotResponse<T> {
    pub response: Option<T>,
    pub error: Option<serde_json::Value> //model kraken errors later
}

impl<Response: for<'a> serde::Deserialize<'a>> SpotRequest<Response> {
    fn new(request: http::Request<String>) -> Self {
        SpotRequest(request,std::marker::PhantomData)
    }
    pub async fn send<F, R, E>(self, func: F) -> Result<Response, Box<dyn std::error::Error>>
    where F: Fn(http::Request<String>) -> R,
        R: std::future::Future<Output = Result<bytes::Bytes, E>>,
        Box<dyn std::error::Error>: From<E>
    {
        let response: SpotResponse<Response> = serde_json::from_slice(&func(self.0).await?)?;
        match response.response {
            Some(data) => Ok(data),
            None => Err(response.error.unwrap().to_string().into())
        }
    }
}
*/
