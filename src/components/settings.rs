use crate::services::config_service::ConfigService;
use crate::types::api_types::AppConfig;
use leptos::*;

#[component]
pub fn Settings() -> impl IntoView {
    let config_service = ConfigService::new();
    let initial_config = config_service.get_config().unwrap_or_default();

    // API配置
    let (deeplx_url, set_deeplx_url) = create_signal(initial_config.deeplx_api_url.clone());
    let (jina_url, set_jina_url) = create_signal(initial_config.jina_api_url.clone());
    let (source_lang, set_source_lang) = create_signal(initial_config.default_source_lang.clone());
    let (target_lang, set_target_lang) = create_signal(initial_config.default_target_lang.clone());
    
    // 速率限制配置
    let (max_requests_per_second, set_max_requests_per_second) = create_signal(initial_config.max_requests_per_second);
    let (max_text_length, set_max_text_length) = create_signal(initial_config.max_text_length);
    let (max_paragraphs_per_request, set_max_paragraphs_per_request) = create_signal(initial_config.max_paragraphs_per_request);
    
    // 文件命名配置
    let (filename_max_length, set_filename_max_length) = create_signal(initial_config.file_naming.max_length);
    let (convert_to_lowercase, set_convert_to_lowercase) = create_signal(initial_config.file_naming.lowercase);
    
    let (save_message, set_save_message) = create_signal(String::new());

    // 生成文件名预览
    let filename_preview = move || {
        let sample_title = "Getting Started Guide";
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let mut preview = format!("{}_{}.md", sample_title.replace(' ', "_"), timestamp);
        
        if convert_to_lowercase.get() {
            preview = preview.to_lowercase();
        }
        
        let max_len = filename_max_length.get();
        if preview.len() > max_len {
            let extension = ".md";
            let base_len = max_len.saturating_sub(extension.len());
            preview = format!("{}...{}", &preview[..base_len.min(preview.len())], extension);
        }
        
        preview
    };

    let save_settings = move |_| {
        let mut config = AppConfig {
            deeplx_api_url: deeplx_url.get(),
            jina_api_url: jina_url.get(),
            default_source_lang: source_lang.get(),
            default_target_lang: target_lang.get(),
            max_requests_per_second: max_requests_per_second.get(),
            max_text_length: max_text_length.get(),
            max_paragraphs_per_request: max_paragraphs_per_request.get(),
            file_naming: initial_config.file_naming.clone(),
        };

        // 更新文件命名配置
        config.file_naming.max_length = filename_max_length.get();
        config.file_naming.lowercase = convert_to_lowercase.get();

        match config_service.save_config(&config) {
            Ok(_) => set_save_message.set("设置保存成功！".to_string()),
            Err(e) => set_save_message.set(format!("保存失败: {e}")),
        }
    };

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            <h2 class="text-2xl font-bold text-gray-800 mb-6">"设置"</h2>
            
            <div class="space-y-8">
                // API配置部分
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "DeepLX API URL"
                        </label>
                        <input
                            type="url"
                            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            placeholder="https://deepl3.filework.online/dptra"
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

                // 速率限制设置
                <div class="border-t pt-6">
                    <h3 class="text-lg font-medium text-gray-800 mb-4">"速率限制设置"</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "每秒最大请求数"
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="100"
                                class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                prop:value=max_requests_per_second
                                on:input=move |ev| {
                                    if let Ok(value) = event_target_value(&ev).parse::<u32>() {
                                        set_max_requests_per_second.set(value);
                                    }
                                }
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "每次请求最大文本长度"
                            </label>
                            <input
                                type="number"
                                min="1000"
                                max="50000"
                                class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                prop:value=max_text_length
                                on:input=move |ev| {
                                    if let Ok(value) = event_target_value(&ev).parse::<usize>() {
                                        set_max_text_length.set(value);
                                    }
                                }
                            />
                            <p class="text-xs text-gray-500 mt-1">
                                "超过此长度的文本将从空行处智能分割"
                            </p>
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "每次请求最大段落数"
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="100"
                                class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                prop:value=max_paragraphs_per_request
                                on:input=move |ev| {
                                    if let Ok(value) = event_target_value(&ev).parse::<usize>() {
                                        set_max_paragraphs_per_request.set(value);
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>

                // 文件命名设置
                <div class="border-t pt-6">
                    <h3 class="text-lg font-medium text-gray-800 mb-4">"文件命名设置"</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "命名模式"
                            </label>
                            <select
                                class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                disabled
                            >
                                <option value="title_timestamp">"标题+时间戳"</option>
                            </select>
                            <p class="text-xs text-gray-500 mt-1">
                                "选择文件命名的格式"
                            </p>
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "文件名最大长度"
                            </label>
                            <input
                                type="number"
                                min="20"
                                max="255"
                                class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                prop:value=filename_max_length
                                on:input=move |ev| {
                                    if let Ok(value) = event_target_value(&ev).parse::<usize>() {
                                        set_filename_max_length.set(value);
                                    }
                                }
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "单词分隔符"
                            </label>
                            <select
                                class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                disabled
                            >
                                <option value="underscore">"下划线 (_)"</option>
                            </select>
                        </div>

                        <div class="flex items-center space-x-2">
                            <input
                                type="checkbox"
                                id="convert_to_lowercase"
                                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                                prop:checked=convert_to_lowercase
                                on:change=move |ev| {
                                    set_convert_to_lowercase.set(event_target_checked(&ev));
                                }
                            />
                            <label for="convert_to_lowercase" class="text-sm font-medium text-gray-700">
                                "转换为小写"
                            </label>
                        </div>
                    </div>

                    <div class="mt-4 p-4 bg-gray-50 rounded-md">
                        <h4 class="text-sm font-medium text-gray-700 mb-2">"预览效果"</h4>
                        <code class="text-sm text-blue-600 bg-white px-2 py-1 rounded border">
                            {move || filename_preview()}
                        </code>
                    </div>
                </div>

                // 主题选择
                <div class="border-t pt-6">
                    <h3 class="text-lg font-medium text-gray-800 mb-4">"主题选择"</h3>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div class="flex items-center space-x-2 p-3 border rounded-md">
                            <div class="w-4 h-4 bg-orange-400 rounded-full"></div>
                            <div class="w-4 h-4 bg-gray-600 rounded-full"></div>
                            <span class="text-sm text-gray-700">"蜜桃 (浅色)"</span>
                        </div>
                        <div class="flex items-center space-x-2 p-3 border rounded-md">
                            <div class="w-4 h-4 bg-orange-400 rounded-full"></div>
                            <div class="w-4 h-4 bg-gray-900 rounded-full"></div>
                            <span class="text-sm text-gray-700">"法拉第 (深色)"</span>
                        </div>
                        <div class="flex items-center space-x-2 p-3 border rounded-md">
                            <div class="w-4 h-4 bg-orange-400 rounded-full"></div>
                            <div class="w-4 h-4 bg-gray-900 rounded-full"></div>
                            <span class="text-sm text-gray-700">"玛奇朵 (深色)"</span>
                        </div>
                        <div class="flex items-center space-x-2 p-3 border rounded-md">
                            <div class="w-4 h-4 bg-orange-400 rounded-full"></div>
                            <div class="w-4 h-4 bg-gray-900 rounded-full"></div>
                            <span class="text-sm text-gray-700">"摩卡 (极深)"</span>
                        </div>
                    </div>
                </div>

                // 保存设置按钮
                <div class="flex items-center justify-between border-t pt-6">
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
            </div>
        </div>
    }
}
