use crate::SERVER_URL;
use gloo_net::http::Request;
use gloo_net::Error;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use yew::prelude::*;
use yew::suspense::use_future;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>
            <NavBar/>
            <main>
                <Suspense fallback={html!{<div aria-busy="true" class="secondary"></div>}}>
                    <Rooms/>
                </Suspense>
            </main>
        </>
    }
}

#[function_component]
fn NavBar() -> Html {
    html! {
        <nav>
            <ul>
                <li><strong>{ "Web Imgage Manager" }</strong></li>
            </ul>
            <div>
                <li><a href="#" role="button" class="secondary outline">{ "Images" }</a></li>
                <li><a href="#" role="button" class="secondary outline">{ "Statistics" }</a></li>
            </div>
        </nav>
    }
}

#[function_component]
fn Rooms() -> HtmlResult {
    let res = use_future(|| async { get_json("list/6a766d31-71d5-4a34-8df5-124b9614b19f").await })?;
    let res: &Vec<Uuid> = match *res {
        Ok(ref res) => res,
        Err(ref err) => return Ok(html! { err.to_string() }),
    };

    Ok(html! {
        <article>
        { for res.iter().map(|item| html! {
            <details>
                <summary>{item.to_string()}</summary>
            </details>
        }) }
        </article>
    })
}

async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, Error> {
    Request::get(&format!("{SERVER_URL}/{url}"))
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<T>()
        .await
}
