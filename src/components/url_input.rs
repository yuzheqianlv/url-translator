use leptos::*;
use crate::theme::use_theme_context;

#[component]
pub fn UrlInput(
    url: ReadSignal<String>,
    set_url: WriteSignal<String>,
    on_submit: impl Fn(web_sys::MouseEvent) + 'static + Copy,
    is_loading: ReadSignal<bool>,
) -> impl IntoView {
    let theme_context = use_theme_context();
    
    let handle_keypress = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" && !is_loading.get() {
            // 创建一个模拟的鼠标事件
            if let Ok(mouse_event) = web_sys::MouseEvent::new("click") {
                on_submit(mouse_event);
            }
        }
    };
    
    view! {
        <div>
            <label class="block text-sm font-medium mb-2" style=move || theme_context.get().theme.text_style()>
                "输入URL"
            </label>
            <div class="flex space-x-2">
                <input
                    type="url"
                    class="flex-1 px-4 py-2 rounded-md focus:ring-2 focus:border-transparent"
                    style=move || theme_context.get().theme.input_style()
                    placeholder="https://example.com"
                    prop:value=url
                    on:input=move |ev| {
                        set_url.set(event_target_value(&ev));
                    }
                    on:keypress=handle_keypress
                    disabled=is_loading
                />
                <button
                    class="px-6 py-2 rounded-md disabled:opacity-50 disabled:cursor-not-allowed transition-colors hover:opacity-90"
                    style=move || theme_context.get().theme.button_primary_style()
                    disabled=is_loading
                    on:click=on_submit
                >
                    {move || if is_loading.get() { "处理中..." } else { "翻译" }}
                </button>
            </div>
        </div>
    }
}