use crate::components::ThemeSelector;
use crate::hooks::use_config::use_config;
use crate::services::file_naming_service::{FileNamingConfig, FileNamingMode};
use crate::theme::use_theme_context;
use crate::types::api_types::AppConfig;
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let config_hook = use_config();
    let theme_context = use_theme_context();

    // 本地状态用于表单编辑
    let (deeplx_url, set_deeplx_url) = create_signal(String::new());
    let (jina_url, set_jina_url) = create_signal(String::new());
    let (source_lang, set_source_lang) = create_signal(String::new());
    let (target_lang, set_target_lang) = create_signal(String::new());
    let (max_requests_per_second, set_max_requests_per_second) = create_signal(String::new());
    let (max_text_length, set_max_text_length) = create_signal(String::new());
    let (max_paragraphs, set_max_paragraphs) = create_signal(String::new());
    let (save_message, set_save_message) = create_signal(String::new());

    // 文件命名配置状态
    let (naming_mode, set_naming_mode) = create_signal(String::new());
    let (max_filename_length, set_max_filename_length) = create_signal(String::new());
    let (use_lowercase, set_use_lowercase) = create_signal(false);
    let (word_separator, set_word_separator) = create_signal(String::new());

    // 初始化表单值
    create_effect(move |_| {
        let config = config_hook.config.get();
        set_deeplx_url.set(config.deeplx_api_url);
        set_jina_url.set(config.jina_api_url);
        set_source_lang.set(config.default_source_lang);
        set_target_lang.set(config.default_target_lang);
        set_max_requests_per_second.set(config.max_requests_per_second.to_string());
        set_max_text_length.set(config.max_text_length.to_string());
        set_max_paragraphs.set(config.max_paragraphs_per_request.to_string());

        // 初始化文件命名配置
        let mode_str = match config.file_naming.mode {
            FileNamingMode::TitleOnly => "title_only",
            FileNamingMode::TitleWithTimestamp => "title_with_timestamp",
            FileNamingMode::DomainPathTitle => "domain_path_title",
            FileNamingMode::OrderTitle => "order_title",
            FileNamingMode::Custom(_) => "custom",
        }
        .to_string();
        set_naming_mode.set(mode_str);
        set_max_filename_length.set(config.file_naming.max_length.to_string());
        set_use_lowercase.set(config.file_naming.lowercase);
        set_word_separator.set(config.file_naming.word_separator);
    });

    let save_settings = move |_| {
        let max_requests_val = max_requests_per_second.get().parse::<u32>().unwrap_or(10);
        let max_text_val = max_text_length.get().parse::<usize>().unwrap_or(5000);
        let max_paragraphs_val = max_paragraphs.get().parse::<usize>().unwrap_or(10);

        // 构建文件命名配置
        let naming_mode_val = match naming_mode.get().as_str() {
            "title_only" => FileNamingMode::TitleOnly,
            "title_with_timestamp" => FileNamingMode::TitleWithTimestamp,
            "domain_path_title" => FileNamingMode::DomainPathTitle,
            "order_title" => FileNamingMode::OrderTitle,
            _ => FileNamingMode::TitleWithTimestamp,
        };

        let max_length_val = max_filename_length.get().parse::<usize>().unwrap_or(100);

        let file_naming_config = FileNamingConfig {
            mode: naming_mode_val,
            max_length: max_length_val,
            timestamp_format: "%Y%m%d_%H%M%S".to_string(),
            remove_special_chars: true,
            lowercase: use_lowercase.get(),
            word_separator: word_separator.get(),
            include_extension: true,
        };

        let new_config = AppConfig {
            deeplx_api_url: deeplx_url.get(),
            jina_api_url: jina_url.get(),
            default_source_lang: source_lang.get(),
            default_target_lang: target_lang.get(),
            max_requests_per_second: max_requests_val,
            max_text_length: max_text_val,
            max_paragraphs_per_request: max_paragraphs_val,
            file_naming: file_naming_config,
        };

        (config_hook.save_config)(new_config);
        set_save_message.set("设置保存成功！".to_string());

        // 3秒后清除消息
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            set_save_message.set(String::new());
        });
    };

    let reset_settings = move |_| {
        (config_hook.reset_config)();
        set_save_message.set("设置已重置为默认值！".to_string());

        // 3秒后清除消息
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            set_save_message.set(String::new());
        });
    };

    view! {
        <div class="max-w-2xl mx-auto">
            <h1 class="text-2xl font-bold mb-6" style=move || theme_context.get().theme.text_style()>
                "设置"
            </h1>

            <div class="rounded-lg shadow-lg p-6" style=move || theme_context.get().theme.card_style()>
                <div class="space-y-6">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <ConfigInput
                            label="DeepLX API URL"
                            placeholder="https://deepl3.fileaiwork.online/dptrans?token=..."
                            value=deeplx_url
                            set_value=set_deeplx_url
                            input_type="url"
                        />

                        <ConfigInput
                            label="Jina API URL"
                            placeholder="https://r.jina.ai"
                            value=jina_url
                            set_value=set_jina_url
                            input_type="url"
                        />

                        <LanguageSelect
                            label="默认源语言"
                            value=source_lang
                            set_value=set_source_lang
                            include_auto=true
                        />

                        <LanguageSelect
                            label="默认目标语言"
                            value=target_lang
                            set_value=set_target_lang
                            include_auto=false
                        />
                    </div>

                    <div class="border-t pt-6 themed-border-t">
                        <h3 class="text-lg font-medium themed-text mb-4">
                            "速率限制设置"
                        </h3>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                            <ConfigInput
                                label="每秒最大请求数"
                                placeholder="10"
                                value=max_requests_per_second
                                set_value=set_max_requests_per_second
                                input_type="number"
                                min="1"
                                max="50"
                            />

                            <ConfigInput
                                label="每次请求最大文本长度"
                                placeholder="5000"
                                value=max_text_length
                                set_value=set_max_text_length
                                input_type="number"
                                min="1000"
                                max="10000"
                            />

                            <ConfigInput
                                label="每次请求最大段落数"
                                placeholder="10"
                                value=max_paragraphs
                                set_value=set_max_paragraphs
                                input_type="number"
                                min="5"
                                max="50"
                            />
                        </div>
                    </div>

                    <div class="border-t pt-6" style=move || format!("border-color: {};", theme_context.get().theme.surface2)>
                        <h3 class="text-lg font-medium themed-text mb-4">
                            "文件命名设置"
                        </h3>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div>
                                <label class="block text-sm font-medium mb-2" style=move || theme_context.get().theme.text_style()>
                                    "命名模式"
                                </label>
                                <select
                                    class="w-full px-4 py-2 rounded-md focus:ring-2 focus:border-transparent"
                                    style=move || theme_context.get().theme.input_style()
                                    prop:value=naming_mode
                                    on:change=move |ev| {
                                        set_naming_mode.set(event_target_value(&ev));
                                    }
                                >
                                    <option value="title_only">"仅标题"</option>
                                    <option value="title_with_timestamp">"标题+时间戳"</option>
                                    <option value="domain_path_title">"域名+路径+标题"</option>
                                    <option value="order_title">"序号+标题"</option>
                                </select>
                                <p class="text-xs mt-1" style=move || theme_context.get().theme.subtext_style()>
                                    "选择文件命名的格式"
                                </p>
                            </div>

                            <ConfigInput
                                label="文件名最大长度"
                                placeholder="100"
                                value=max_filename_length
                                set_value=set_max_filename_length
                                input_type="number"
                                min="20"
                                max="200"
                            />

                            <div>
                                <label class="block text-sm font-medium mb-2" style=move || theme_context.get().theme.text_style()>
                                    "单词分隔符"
                                </label>
                                <select
                                    class="w-full px-4 py-2 rounded-md focus:ring-2 focus:border-transparent"
                                    style=move || theme_context.get().theme.input_style()
                                    prop:value=word_separator
                                    on:change=move |ev| {
                                        set_word_separator.set(event_target_value(&ev));
                                    }
                                >
                                    <option value="_">"下划线 (_)"</option>
                                    <option value="-">"连字符 (-)"</option>
                                    <option value=" ">"空格"</option>
                                </select>
                            </div>

                            <div class="flex items-center">
                                <input
                                    type="checkbox"
                                    id="use_lowercase"
                                    class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                    prop:checked=use_lowercase
                                    on:change=move |ev| {
                                        let target = ev.target().unwrap();
                                        let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                        set_use_lowercase.set(input.checked());
                                    }
                                />
                                <label
                                    for="use_lowercase"
                                    class="ml-2 text-sm font-medium"
                                    style=move || theme_context.get().theme.text_style()
                                >
                                    "转换为小写"
                                </label>
                            </div>
                        </div>

                        <div class="mt-4 p-3 rounded-md" style=move || theme_context.get().theme.content_bg_style()>
                            <h4 class="text-sm font-medium mb-2" style=move || theme_context.get().theme.text_style()>
                                "预览效果"
                            </h4>
                            <div class="font-mono text-sm" style=move || theme_context.get().theme.subtext_style()>
                                {move || {
                                    let mode = naming_mode.get();
                                    let separator = word_separator.get();
                                    let lowercase = use_lowercase.get();

                                    let sample = match mode.as_str() {
                                        "title_only" => "getting_started_guide",
                                        "title_with_timestamp" => "getting_started_guide_20250708_143022",
                                        "domain_path_title" => "example_com_docs_getting_started_guide",
                                        "order_title" => "001_getting_started_guide",
                                        _ => "getting_started_guide_20250708_143022",
                                    };

                                    let processed = if lowercase {
                                        sample.to_lowercase()
                                    } else {
                                        sample.to_string()
                                    };

                                    let final_name = if separator != "_" {
                                        processed.replace("_", &separator)
                                    } else {
                                        processed
                                    };

                                    format!("{}.md", final_name)
                                }}
                            </div>
                        </div>
                    </div>

                    <div class="border-t pt-6" style=move || format!("border-color: {};", theme_context.get().theme.surface2)>
                        <ThemeSelector />
                    </div>

                    <div class="flex items-center justify-between border-t pt-6" style=move || format!("border-color: {};", theme_context.get().theme.surface2)>
                        <div class="flex space-x-3">
                            <button
                                class="px-6 py-2 rounded-md transition-colors"
                                class:opacity-50=config_hook.is_loading
                                style=move || theme_context.get().theme.button_primary_style()
                                disabled=config_hook.is_loading
                                on:click=save_settings
                            >
                                {move || if config_hook.is_loading.get() { "保存中..." } else { "保存设置" }}
                            </button>

                            <button
                                class="px-6 py-2 rounded-md transition-colors"
                                style=move || theme_context.get().theme.button_secondary_style()
                                disabled=config_hook.is_loading
                                on:click=reset_settings
                            >
                                "重置为默认"
                            </button>
                        </div>

                        <Show when=move || !save_message.get().is_empty()>
                            <div class="font-medium" style=move || format!("color: {};", theme_context.get().theme.success_color())>
                                {save_message}
                            </div>
                        </Show>
                    </div>

                    <div class="border-t pt-6 themed-border-t">
                        <h3 class="text-lg font-medium themed-text mb-3">
                            "使用说明"
                        </h3>
                        <div class="text-sm space-y-2 themed-subtext">
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

#[component]
fn ConfigInput(
    label: &'static str,
    placeholder: &'static str,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    input_type: &'static str,
    #[prop(optional)] min: &'static str,
    #[prop(optional)] max: &'static str,
) -> impl IntoView {
    let theme_context = use_theme_context();

    view! {
        <div>
            <label class="block text-sm font-medium mb-2" style=move || theme_context.get().theme.text_style()>
                {label}
            </label>
            <input
                type=input_type
                class="w-full px-4 py-2 rounded-md focus:ring-2 focus:border-transparent"
                style=move || theme_context.get().theme.input_style()
                placeholder=placeholder
                prop:value=value
                prop:min=min
                prop:max=max
                on:input=move |ev| {
                    set_value.set(event_target_value(&ev));
                }
            />
        </div>
    }
}

#[component]
fn LanguageSelect(
    label: &'static str,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    include_auto: bool,
) -> impl IntoView {
    let theme_context = use_theme_context();

    view! {
        <div>
            <label class="block text-sm font-medium mb-2" style=move || theme_context.get().theme.text_style()>
                {label}
            </label>
            <select
                class="w-full px-4 py-2 rounded-md focus:ring-2 focus:border-transparent"
                style=move || theme_context.get().theme.input_style()
                prop:value=value
                on:change=move |ev| {
                    set_value.set(event_target_value(&ev));
                }
            >
                {if include_auto {
                    view! { <option value="auto">"自动检测"</option> }.into_view()
                } else {
                    view! {}.into_view()
                }}
                <option value="ZH">"中文"</option>
                <option value="EN">"英语"</option>
                <option value="JA">"日语"</option>
                <option value="FR">"法语"</option>
                <option value="DE">"德语"</option>
                <option value="ES">"西班牙语"</option>
            </select>
        </div>
    }
}
