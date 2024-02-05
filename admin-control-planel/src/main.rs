mod app;

use app::App;

const SERVER_URL: &str = "";

fn main() {
    yew::Renderer::<App>::new().render();
}
