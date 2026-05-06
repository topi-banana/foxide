fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<foxide_frontend::App>::new().render();
}
