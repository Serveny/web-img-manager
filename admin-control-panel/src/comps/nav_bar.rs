use yew::prelude::*;

#[function_component]
pub fn NavBar() -> Html {
    html! {
        <nav>
            <ul>
                <li><strong>{ "Web Imgage Manager" }</strong></li>
            </ul>
            <ul>
                <li><a href="#statistics">{ "Statistics" }</a></li>
                <li><a href="#images">{ "Images" }</a></li>
            </ul>
        </nav>
    }
}
