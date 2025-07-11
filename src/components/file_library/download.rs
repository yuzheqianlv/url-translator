//! Download functionality for translation files

use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

use crate::types::api_types::TranslationResponse;
use crate::components::file_library::view_mode::ViewMode;

/// Download button component
#[component]
pub fn DownloadButton(
    /// Translation data
    translation: ReadSignal<Option<TranslationResponse>>,
    /// Current view mode
    view_mode: ReadSignal<ViewMode>,
    /// Base file name
    file_name: impl Fn() -> String + 'static,
) -> impl IntoView {
    let on_download = move |_| {
        if let Some(trans) = translation.get() {
            let content = generate_download_content(&trans, view_mode.get());
            let filename = generate_filename(&file_name(), view_mode.get());
            download_file(&content, &filename);
        }
    };

    view! {
        <button
            class="flex items-center px-3 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            on:click=on_download
            disabled=move || translation.get().is_none()
            title=move || format!("下载 {} 格式", view_mode.get().to_string())
        >
            <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V15a2 2 0 01-2 2z"/>
            </svg>
            "下载"
        </button>
    }
}

/// Compact download button for smaller spaces
#[component]
pub fn CompactDownloadButton(
    /// Translation data
    translation: ReadSignal<Option<TranslationResponse>>,
    /// Current view mode
    view_mode: ReadSignal<ViewMode>,
    /// Base file name
    file_name: impl Fn() -> String + 'static,
) -> impl IntoView {
    let on_download = move |_| {
        if let Some(trans) = translation.get() {
            let content = generate_download_content(&trans, view_mode.get());
            let filename = generate_filename(&file_name(), view_mode.get());
            download_file(&content, &filename);
        }
    };

    view! {
        <button
            class="p-2 text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded transition-colors"
            on:click=on_download
            disabled=move || translation.get().is_none()
            title=move || format!("下载 {} 格式", view_mode.get().to_string())
        >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V15a2 2 0 01-2 2z"/>
            </svg>
        </button>
    }
}

/// Download dropdown with multiple format options
#[component]
pub fn DownloadDropdown(
    /// Translation data
    translation: ReadSignal<Option<TranslationResponse>>,
    /// Base file name
    file_name: impl Fn() -> String + 'static,
) -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    let download_option = move |mode: ViewMode| {
        move |_| {
            if let Some(trans) = translation.get() {
                let content = generate_download_content(&trans, mode);
                let filename = generate_filename(&file_name(), mode);
                download_file(&content, &filename);
                set_is_open.set(false);
            }
        }
    };

    view! {
        <div class="relative">
            <button
                class="flex items-center px-3 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                on:click=move |_| set_is_open.update(|open| *open = !*open)
                disabled=move || translation.get().is_none()
            >
                <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V15a2 2 0 01-2 2z"/>
                </svg>
                "下载"
                <svg class="h-4 w-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
            </button>

            <Show when=is_open>
                <div class="absolute right-0 mt-2 w-48 bg-white rounded-lg shadow-lg border border-gray-200 z-50">
                    <div class="py-1">
                        <button
                            class="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            on:click=download_option(ViewMode::Original)
                        >
                            <svg class="h-4 w-4 mr-3 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                            </svg>
                            "下载原文"
                        </button>
                        <button
                            class="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            on:click=download_option(ViewMode::Translated)
                        >
                            <svg class="h-4 w-4 mr-3 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"/>
                            </svg>
                            "下载译文"
                        </button>
                        <button
                            class="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            on:click=download_option(ViewMode::Bilingual)
                        >
                            <svg class="h-4 w-4 mr-3 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2v0a2 2 0 01-2-2V9a2 2 0 00-2-2H8z"/>
                            </svg>
                            "下载双语对照"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Generate filename based on view mode
fn generate_filename(base_name: &str, view_mode: ViewMode) -> String {
    let mode_suffix = match view_mode {
        ViewMode::Original => "original",
        ViewMode::Translated => "translated",
        ViewMode::Bilingual => "bilingual",
    };
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let clean_name = base_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();
    
    format!("{}_{}__{}.md", clean_name, mode_suffix, timestamp)
}

/// Generate content based on view mode
fn generate_download_content(translation: &TranslationResponse, view_mode: ViewMode) -> String {
    let timestamp = translation.created_at.format("%Y-%m-%d %H:%M:%S UTC");
    let header = format!(
        "---\n\
        title: {}\n\
        url: {}\n\
        source_language: {}\n\
        target_language: {}\n\
        translation_date: {}\n\
        view_mode: {}\n\
        ---\n\n",
        translation.url.split('/').last().unwrap_or("translation"),
        translation.url,
        translation.source_lang,
        translation.target_lang,
        timestamp,
        view_mode.to_string()
    );

    match view_mode {
        ViewMode::Original => {
            format!(
                "{}\
                # 原始内容\n\n\
                > 来源: {}\n\
                > 提取时间: {}\n\n\
                {}\n",
                header,
                translation.url,
                timestamp,
                translation.original_content
            )
        }
        ViewMode::Translated => {
            format!(
                "{}\
                # 翻译内容\n\n\
                > 来源: {}\n\
                > 翻译时间: {}\n\
                > 语言: {} → {}\n\n\
                {}\n",
                header,
                translation.url,
                timestamp,
                translation.source_lang,
                translation.target_lang,
                translation.translated_content
            )
        }
        ViewMode::Bilingual => {
            // Split content into paragraphs for bilingual display
            let original_paragraphs: Vec<&str> = translation.original_content
                .split("\n\n")
                .filter(|p| !p.trim().is_empty())
                .collect();
            
            let translated_paragraphs: Vec<&str> = translation.translated_content
                .split("\n\n")
                .filter(|p| !p.trim().is_empty())
                .collect();

            let mut content = format!(
                "{}\
                # 双语对照\n\n\
                > 来源: {}\n\
                > 翻译时间: {}\n\
                > 语言: {} → {}\n\n",
                header,
                translation.url,
                timestamp,
                translation.source_lang,
                translation.target_lang
            );

            let max_paragraphs = original_paragraphs.len().max(translated_paragraphs.len());
            
            for i in 0..max_paragraphs {
                content.push_str(&format!("## 段落 {}\n\n", i + 1));
                
                if let Some(original) = original_paragraphs.get(i) {
                    content.push_str("### 原文\n\n");
                    content.push_str(original);
                    content.push_str("\n\n");
                }
                
                if let Some(translated) = translated_paragraphs.get(i) {
                    content.push_str("### 译文\n\n");
                    content.push_str(translated);
                    content.push_str("\n\n");
                }
                
                content.push_str("---\n\n");
            }

            content
        }
    }
}

/// Download file using browser's download mechanism
fn download_file(content: &str, filename: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            // Create blob
            let mut blob_options = BlobPropertyBag::new();
            blob_options.type_("text/markdown;charset=utf-8");
            
            let parts = js_sys::Array::new();
            parts.push(&wasm_bindgen::JsValue::from_str(content));
            
            if let Ok(blob) = Blob::new_with_str_sequence_and_options(&parts, &blob_options) {
                // Create download URL
                if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                    // Create temporary anchor element
                    if let Ok(anchor) = document.create_element("a") {
                        let anchor = anchor.dyn_into::<HtmlAnchorElement>().unwrap();
                        anchor.set_href(&url);
                        anchor.set_download(filename);
                        anchor.set_style("display: none");
                        
                        // Add to document, click, and remove
                        if let Some(body) = document.body() {
                            let _ = body.append_child(&anchor);
                            anchor.click();
                            let _ = body.remove_child(&anchor);
                        }
                        
                        // Clean up the URL
                        Url::revoke_object_url(&url).ok();
                    }
                }
            }
        }
    }
}