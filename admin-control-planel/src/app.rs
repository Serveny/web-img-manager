use crate::SERVER_URL;
use gloo_net::http::Request;
use uuid::Uuid;
use yew::prelude::*;
use yew::suspense::use_future;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <img class="logo" src="https://yew.rs/img/logo.png" alt="Yew logo" />
            <h1>{ "Hello World!" }</h1>
            <span class="subtitle">{ "from Yew with " }<i class="heart" /></span>
            <Rooms></Rooms>
        </main>
    }
}

#[function_component]
fn Rooms() -> HtmlResult {
    let res = use_future(|| async {
        Request::get(&format!(
            "{SERVER_URL}/list/6a766d31-71d5-4a34-8df5-124b9614b19f"
        ))
        .send()
        .await?
        .text()
        .await
    })?;
    let res: &String = match *res {
        Ok(ref res) => res,
        Err(ref err) => return Ok(html! { err.to_string() }),
    };
    let res: Vec<Uuid> = match serde_json::from_str(res) {
        Ok(res) => res,
        Err(err) => return Ok(html! { err.to_string() }),
    };

    Ok(html! {})
}
