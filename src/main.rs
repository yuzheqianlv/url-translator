use leptos::mount_to_body;

mod app;
mod components;
mod services;
mod types;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    
    mount_to_body(App);
}
