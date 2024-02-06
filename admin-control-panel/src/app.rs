use crate::comps::{ImagesCard, NavBar, StatisticsCard};
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>
            <NavBar/>
            <Suspense fallback={html!{<div aria-busy="true" class="secondary"></div>}}>
                <StatisticsCard />
                <ImagesCard />
            </Suspense>
        </>
    }
}
