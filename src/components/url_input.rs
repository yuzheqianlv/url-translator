use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::{jina_service::JinaService, deeplx_service::DeepLXService, config_service::ConfigService};

#[component]
pub fn UrlInput() -> impl IntoView {
    let (url, set_url) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(String::new());
    
    let set_translation_result = use_context::<WriteSignal<String>>()
        .expect("Translation result setter context not found");
    
    let handle_translate = move |_| {
        let url_value = url.get();
        if url_value.is_empty() {
            set_error_message.set("请输入有效的URL".to_string());
            return;
        }
        
        set_is_loading.set(true);
        set_error_message.set(String::new());
        
        spawn_local(async move {
            let config_service = ConfigService::new();
            
            match config_service.get_config() {
                Ok(config) => {
                    let jina_service = JinaService::new(&config);
                    let deeplx_service = DeepLXService::new(&config);
                    
                    match jina_service.extract_content(&url_value, &config).await {
                        Ok(content) => {
                            match deeplx_service.translate(
                                &content,
                                &config.default_source_lang,
                                &config.default_target_lang,
                                &config,
                            ).await {
                                Ok(translated) => {
                                    set_translation_result.set(translated);
                                }
                                Err(e) => {
                                    set_error_message.set(format!("翻译失败: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            set_error_message.set(format!("内容提取失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    set_error_message.set(format!("配置加载失败: {}", e));
                }
            }
            
            set_is_loading.set(false);
        });
    };
    
    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            <div class="space-y-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        "输入URL"
                    </label>
                    <input
                        type="url"
                        class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        placeholder="https://example.com"
                        prop:value=url
                        on:input=move |ev| {
                            set_url.set(event_target_value(&ev));
                        }
                    />
                </div>
                
                <button
                    class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled=is_loading
                    on:click=handle_translate
                >
                    {move || if is_loading.get() { "处理中..." } else { "开始翻译" }}
                </button>
                
                {move || {
                    let error = error_message.get();
                    if !error.is_empty() {
                        view! {
                            <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                                {error}
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }
                }}
            </div>
        </div>
    }
}