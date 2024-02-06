use crate::SERVER_URL;
use gloo_net::http::Request;
use gloo_net::Error;
use serde::de::DeserializeOwned;

pub async fn get_json<T: DeserializeOwned>(url: String) -> Result<T, Error> {
    Request::get(&format!("{SERVER_URL}/{url}"))
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<T>()
        .await
}
