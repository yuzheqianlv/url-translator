use leptos::mount_to_body;

mod app;
mod components;
mod error;
mod hooks;
mod pages;
mod services;
mod theme;
mod types;
mod utils;

use app::App;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(App);
}
