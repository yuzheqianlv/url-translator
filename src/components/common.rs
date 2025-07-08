use leptos::*;

#[component]
pub fn ProgressIndicator(
    #[prop(into)] progress: Signal<f32>,
    #[prop(into, optional)] message: Signal<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <div class=format!("w-full {}", class)>
            <Show when=move || !message.get().is_empty()>
                <div class="mb-2 text-sm text-gray-600 dark:text-gray-400">
                    {message}
                </div>
            </Show>
            <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div 
                    class="bg-blue-600 dark:bg-blue-400 h-2 rounded-full transition-all duration-300"
                    style:width=move || format!("{}%", progress.get().clamp(0.0, 100.0))
                ></div>
            </div>
        </div>
    }
}