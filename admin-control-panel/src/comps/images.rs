use crate::utils::get_json;
use crate::SERVER_URL;
use uuid::Uuid;
use yew::prelude::*;
use yew::suspense::{use_future, use_future_with};

#[function_component]
pub fn ImagesCard() -> HtmlResult {
    let res = use_future(|| get_json(String::from("list/6a766d31-71d5-4a34-8df5-124b9614b19f")))?;
    let room_ids: &Vec<Uuid> = match *res {
        Ok(ref res) => res,
        Err(ref err) => return Ok(html! { err.to_string() }),
    };

    Ok(html! {
        <article id="images">
        { for room_ids.iter().map(|room_id| html! {
            <Room id={ room_id.clone() } />
        }) }
        </article>
    })
}

#[derive(Properties, PartialEq)]
pub struct RoomProps {
    pub id: Uuid,
}

#[function_component]
fn Room(props: &RoomProps) -> HtmlResult {
    let room_id = props.id;
    let res = use_future_with(room_id, move |r| {
        get_json(format!("list/6a766d31-71d5-4a34-8df5-124b9614b19f/{r}"))
    })?;
    let img_ids: &Vec<u32> = match *res {
        Ok(ref res) => res,
        Err(ref err) => return Ok(html! { err.to_string() }),
    };
    Ok(html! {
        <details open=true>
        <summary>{props.id.to_string()}</summary>
        <div>
            { for img_ids.iter().map(|img_id| html! {
                <img src={ format!("{SERVER_URL}/img/thumb/6a766d31-71d5-4a34-8df5-124b9614b19f/{room_id}/{img_id}") } />
            }) }
        </div>
        </details>
    })
}
