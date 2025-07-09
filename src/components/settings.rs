use crate::services::config_service::ConfigService;
use crate::types::api_types::AppConfig;
use leptos::*;

#[component]
pub fn Settings() -> impl IntoView {
    let config_service = ConfigService::new();
    let initial_config = config_service.get_config().unwrap_or_default();

    let (deeplx_url, set_deeplx_url) = create_signal(initial_config.deeplx_api_url.clone());
    let (jina_url, set_jina_url) = create_signal(initial_config.jina_api_url.clone());
    let (source_lang, set_source_lang) = create_signal(initial_config.default_source_lang.clone());
    let (target_lang, set_target_lang) = create_signal(initial_config.default_target_lang.clone());
    let (save_message, set_save_message) = create_signal(String::new());

    let save_settings = move |_| {
        let config = AppConfig {
            deeplx_api_url: deeplx_url.get(),
            jina_api_url: jina_url.get(),
            default_source_lang: source_lang.get(),
            default_target_lang: target_lang.get(),
            max_requests_per_second: 10,
            max_text_length: 5000,
            max_paragraphs_per_request: 10,
            file_naming: initial_config.file_naming.clone(),
        };

        match config_service.save_config(&config) {
            Ok(_) => set_save_message.set("设置保存成功！".to_string()),
            Err(e) => set_save_message.set(format!("保存失败: {}", e)),
        }
    };

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            <div class="space-y-6">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "DeepLX API URL"
                        </label>
                        <input
                            type="url"
                            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            placeholder="https://api.deeplx.org/translate"
                            prop:value=deeplx_url
                            on:input=move |ev| {
                                set_deeplx_url.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "Jina API URL"
                        </label>
                        <input
                            type="url"
                            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            placeholder="https://r.jina.ai"
                            prop:value=jina_url
                            on:input=move |ev| {
                                set_jina_url.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "默认源语言"
                        </label>
                        <select
                            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            prop:value=source_lang
                            on:change=move |ev| {
                                set_source_lang.set(event_target_value(&ev));
                            }
                        >
                            <option value="auto">"自动检测"</option>
                            <option value="EN">"英语"</option>
                            <option value="ZH">"中文"</option>
                            <option value="JA">"日语"</option>
                            <option value="FR">"法语"</option>
                            <option value="DE">"德语"</option>
                            <option value="ES">"西班牙语"</option>
                        </select>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "默认目标语言"
                        </label>
                        <select
                            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            prop:value=target_lang
                            on:change=move |ev| {
                                set_target_lang.set(event_target_value(&ev));
                            }
                        >
                            <option value="ZH">"中文"</option>
                            <option value="EN">"英语"</option>
                            <option value="JA">"日语"</option>
                            <option value="FR">"法语"</option>
                            <option value="DE">"德语"</option>
                            <option value="ES">"西班牙语"</option>
                        </select>
                    </div>
                </div>

                <div class="flex items-center justify-between">
                    <button
                        class="bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 transition-colors"
                        on:click=save_settings
                    >
                        "保存设置"
                    </button>

                    {move || {
                        let message = save_message.get();
                        if !message.is_empty() {
                            view! {
                                <div class="text-green-600 font-medium">
                                    {message}
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}
                </div>

                <div class="border-t pt-6">
                    <h3 class="text-lg font-medium text-gray-800 mb-3">
                        "使用说明"
                    </h3>
                    <div class="text-sm text-gray-600 space-y-2">
                        <p>"• 输入要翻译的网页URL，系统会自动提取内容并翻译"</p>
                        <p>"• 使用Jina AI Reader服务提取网页内容，保持原始格式"</p>
                        <p>"• 使用DeepLX API进行翻译，支持多种语言"</p>
                        <p>"• 翻译完成后可以下载Markdown格式的文件"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
