use leptos::*;
use leptos_router::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;
use web_sys::{window, Blob, Url};
use crate::services::{jina_service::JinaService, deeplx_service::DeepLXService, config_service::ConfigService};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <Router>
                <AppHeader />
                <main class="container mx-auto px-4 py-8">
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/settings" view=SettingsPage/>
                    </Routes>
                </main>
            </Router>
        </div>
    }
}

#[component]
fn AppHeader() -> impl IntoView {
    view! {
        <header class="bg-white shadow-sm border-b">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center space-x-4">
                        <A href="/" class="text-xl font-bold text-gray-800 hover:text-blue-600">
                            "URL翻译工具"
                        </A>
                    </div>
                    
                    <nav class="flex items-center space-x-6">
                        <A 
                            href="/" 
                            class="text-gray-600 hover:text-blue-600 transition-colors"
                            active_class="text-blue-600 font-medium"
                        >
                            "首页"
                        </A>
                        <A 
                            href="/settings" 
                            class="text-gray-600 hover:text-blue-600 transition-colors"
                            active_class="text-blue-600 font-medium"
                        >
                            "设置"
                        </A>
                    </nav>
                </div>
            </div>
        </header>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (url, set_url) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (translation_result, set_translation_result) = create_signal(String::new());
    let (error_message, set_error_message) = create_signal(String::new());
    let (progress_message, set_progress_message) = create_signal(String::new());
    
    let handle_translate = move |_| {
        let url_value = url.get();
        if url_value.is_empty() {
            set_error_message.set("请输入有效的URL".to_string());
            return;
        }
        
        set_is_loading.set(true);
        set_error_message.set(String::new());
        set_translation_result.set(String::new());
        set_progress_message.set("正在提取网页内容...".to_string());
        
        let set_progress_clone = set_progress_message.clone();
        let set_loading_clone = set_is_loading.clone();
        let set_error_clone = set_error_message.clone();
        let set_result_clone = set_translation_result.clone();
        
        spawn_local(async move {
            // 添加调试日志
            web_sys::console::log_1(&"=== 开始翻译流程 ===".into());
            web_sys::console::log_1(&format!("URL: {}", url_value).into());
            
            let config_service = ConfigService::new();
            web_sys::console::log_1(&"配置服务已创建".into());
            
            match config_service.get_config() {
                Ok(config) => {
                    web_sys::console::log_1(&"配置加载成功，创建服务...".into());
                    let jina_service = JinaService::new(&config);
                    web_sys::console::log_1(&"Jina服务已创建".into());
                    let deeplx_service = DeepLXService::new(&config);
                    web_sys::console::log_1(&"DeepLX服务已创建".into());
                    
                    web_sys::console::log_1(&format!("配置信息: Jina URL: {}, DeepLX URL: {}", 
                        config.jina_api_url, config.deeplx_api_url).into());
                    
                    // 步骤1: 提取内容
                    web_sys::console::log_1(&"=== 步骤1: 开始提取网页内容 ===".into());
                    web_sys::console::log_1(&format!("目标URL: {}", url_value).into());
                    set_progress_clone.set("正在提取网页内容...".to_string());
                    web_sys::console::log_1(&"进度状态已更新为：正在提取网页内容...".into());
                    
                    web_sys::console::log_1(&"开始调用 jina_service.extract_content".into());
                    match jina_service.extract_content(&url_value, &config).await {
                        Ok(content) => {
                            web_sys::console::log_1(&format!("内容提取成功，长度: {} 字符", content.len()).into());
                            
                            // 显示提取的内容前200字符用于调试
                            let preview = if content.len() > 200 {
                                format!("{}...", &content[..200])
                            } else {
                                content.clone()
                            };
                            web_sys::console::log_1(&format!("内容预览: {}", preview).into());
                            
                            // 步骤2: 翻译内容
                            web_sys::console::log_1(&format!("正在翻译内容，从 {} 到 {}", 
                                config.default_source_lang, config.default_target_lang).into());
                            set_progress_clone.set("正在翻译内容...".to_string());
                            
                            match deeplx_service.translate(
                                &content,
                                &config.default_source_lang,
                                &config.default_target_lang,
                                &config,
                            ).await {
                                Ok(translated) => {
                                    web_sys::console::log_1(&"翻译成功！".into());
                                    set_result_clone.set(translated);
                                    set_progress_clone.set("翻译完成！".to_string());
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("翻译失败: {}", e).into());
                                    set_error_clone.set(format!("翻译失败: {}。请检查DeepLX API是否可用。", e));
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("内容提取失败: {}", e).into());
                            set_error_clone.set(format!("内容提取失败: {}。请检查URL是否有效，或Jina API是否可用。", e));
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("配置加载失败: {}", e).into());
                    set_error_clone.set(format!("配置加载失败: {}", e));
                }
            }
            
            set_loading_clone.set(false);
            set_progress_clone.set(String::new());
        });
    };

    let download_markdown = move |_| {
        let content = translation_result.get();
        if content.is_empty() {
            return;
        }
        
        let _ = create_and_download_file(&content, "translated_content.md", "text/markdown");
    };
    
    view! {
        <div class="max-w-4xl mx-auto space-y-6">
            <h1 class="text-3xl font-bold text-center text-gray-800">
                "URL内容翻译工具"
            </h1>
            
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
                        class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
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
                    
                    {move || {
                        let progress = progress_message.get();
                        if !progress.is_empty() && is_loading.get() {
                            view! {
                                <div class="bg-blue-100 border border-blue-400 text-blue-700 px-4 py-3 rounded flex items-center">
                                    <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    {progress}
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}
                </div>
            </div>

            <div class="bg-white rounded-lg shadow-lg p-6">
                <div class="flex justify-between items-center mb-4">
                    <h2 class="text-xl font-semibold text-gray-800">
                        "翻译结果"
                    </h2>
                    <Show when=move || !translation_result.get().is_empty()>
                        <button
                            class="bg-green-600 text-white px-4 py-2 rounded-md hover:bg-green-700 flex items-center space-x-2 transition-colors"
                            on:click=download_markdown
                        >
                            <span>"下载 Markdown"</span>
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                            </svg>
                        </button>
                    </Show>
                </div>
                
                <div class="min-h-[300px] max-h-[600px] overflow-y-auto">
                    <Show
                        when=move || !translation_result.get().is_empty()
                        fallback=|| view! {
                            <div class="flex items-center justify-center h-48 text-gray-500">
                                <div class="text-center">
                                    <svg class="w-12 h-12 mx-auto mb-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                    </svg>
                                    <p class="text-lg font-medium">"暂无翻译内容"</p>
                                    <p class="text-sm">"请输入URL并点击翻译按钮"</p>
                                </div>
                            </div>
                        }
                    >
                        <div class="prose prose-sm max-w-none">
                            <pre class="whitespace-pre-wrap text-sm text-gray-800 leading-relaxed bg-gray-50 p-4 rounded">
                                {move || translation_result.get()}
                            </pre>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SettingsPage() -> impl IntoView {
    let config_service = ConfigService::new();
    let initial_config = config_service.get_config().unwrap_or_default();
    
    let (deeplx_url, set_deeplx_url) = create_signal(initial_config.deeplx_api_url.clone());
    let (jina_url, set_jina_url) = create_signal(initial_config.jina_api_url.clone());
    let (source_lang, set_source_lang) = create_signal(initial_config.default_source_lang.clone());
    let (target_lang, set_target_lang) = create_signal(initial_config.default_target_lang.clone());
    let (max_requests_per_second, set_max_requests_per_second) = create_signal(initial_config.max_requests_per_second.to_string());
    let (max_text_length, set_max_text_length) = create_signal(initial_config.max_text_length.to_string());
    let (max_paragraphs, set_max_paragraphs) = create_signal(initial_config.max_paragraphs_per_request.to_string());
    let (save_message, set_save_message) = create_signal(String::new());
    
    let save_settings = move |_| {
        let max_requests_val = max_requests_per_second.get().parse::<u32>().unwrap_or(5);
        let max_text_val = max_text_length.get().parse::<usize>().unwrap_or(1200);
        let max_paragraphs_val = max_paragraphs.get().parse::<usize>().unwrap_or(5);
        
        let config = crate::types::api_types::AppConfig {
            deeplx_api_url: deeplx_url.get(),
            jina_api_url: jina_url.get(),
            default_source_lang: source_lang.get(),
            default_target_lang: target_lang.get(),
            max_requests_per_second: max_requests_val,
            max_text_length: max_text_val,
            max_paragraphs_per_request: max_paragraphs_val,
        };
        
        match config_service.save_config(&config) {
            Ok(_) => set_save_message.set("设置保存成功！".to_string()),
            Err(e) => set_save_message.set(format!("保存失败: {}", e)),
        }
    };
    
    view! {
        <div class="max-w-2xl mx-auto">
            <h1 class="text-2xl font-bold mb-6 text-gray-800">
                "设置"
            </h1>
            
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
                                placeholder="https://deepl3.fileaiwork.online/dptrans?token=..."
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
                    
                    <div class="border-t pt-6">
                        <h3 class="text-lg font-medium text-gray-800 mb-4">
                            "速率限制设置"
                        </h3>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "每秒最大请求数"
                                </label>
                                <input
                                    type="number"
                                    min="1"
                                    max="50"
                                    class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="10"
                                    prop:value=max_requests_per_second
                                    on:input=move |ev| {
                                        set_max_requests_per_second.set(event_target_value(&ev));
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
                                    max="10000"
                                    class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="5000"
                                    prop:value=max_text_length
                                    on:input=move |ev| {
                                        set_max_text_length.set(event_target_value(&ev));
                                    }
                                />
                            </div>
                            
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "每次请求最大段落数"
                                </label>
                                <input
                                    type="number"
                                    min="5"
                                    max="50"
                                    class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="10"
                                    prop:value=max_paragraphs
                                    on:input=move |ev| {
                                        set_max_paragraphs.set(event_target_value(&ev));
                                    }
                                />
                            </div>
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
        </div>
    }
}

fn create_and_download_file(content: &str, filename: &str, _mime_type: &str) -> Result<(), JsValue> {
    let window = window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;
    
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&JsValue::from_str(content));
    
    let blob = Blob::new_with_str_sequence(&blob_parts)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    
    let anchor = document.create_element("a")?;
    anchor.set_attribute("href", &url)?;
    anchor.set_attribute("download", filename)?;
    anchor.set_attribute("style", "display: none")?;
    
    document.body().unwrap().append_child(&anchor)?;
    
    let html_anchor = anchor.dyn_ref::<web_sys::HtmlAnchorElement>().unwrap();
    html_anchor.click();
    
    document.body().unwrap().remove_child(&anchor)?;
    Url::revoke_object_url(&url)?;
    
    Ok(())
}