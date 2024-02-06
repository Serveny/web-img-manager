mod app;

use app::App;

const SERVER_URL: &str = "http://127.0.0.1:8080";

fn main() {
    yew::Renderer::<App>::new().render();
}
