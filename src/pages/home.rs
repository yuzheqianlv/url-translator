use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, Blob, Url};
use crate::hooks::use_translation::use_translation;
use crate::components::{UrlInput, TranslationResult, ProgressIndicator};

#[component]
pub fn HomePage() -> impl IntoView {
    let translation = use_translation();
    let (url, set_url) = create_signal(String::new());
    
    let handle_translate = move |_| {
        let url_value = url.get();
        translation.translate.set(Some(url_value));
    };
    
    let download_markdown = move |_| {
        let content = translation.translation_result.get();
        if content.is_empty() {
            return;
        }
        
        let _ = create_and_download_file(&content, "translated_content.md", "text/markdown");
    };
    
    view! {
        <div class="max-w-4xl mx-auto space-y-6">
            <div class="text-center">
                <h1 class="text-3xl font-bold themed-text mb-4">
                    "URL内容翻译工具"
                </h1>
                <p class="themed-subtext mb-6">
                    "智能翻译网页内容，支持代码块保护，提供单页和批量翻译模式"
                </p>
            </div>

            // 翻译模式选择卡片
            <div class="grid md:grid-cols-2 gap-6 mb-8">
                // 单页翻译卡片
                <div class="rounded-lg shadow-lg p-6 themed-bg-surface0 border border-transparent hover:border-blue-200 dark:hover:border-blue-700 transition-all">
                    <div class="flex items-center mb-4">
                        <div class="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg mr-3">
                            <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold themed-text">"单页翻译"</h3>
                            <p class="text-sm themed-subtext">"翻译单个网页内容"</p>
                        </div>
                    </div>
                    <p class="themed-subtext text-sm mb-4">
                        "输入网页URL，快速翻译单个页面的内容。支持自动提取正文、保护代码块、生成Markdown文件。"
                    </p>
                    <div class="text-sm text-green-600 dark:text-green-400 font-medium">
                        "✓ 当前模式"
                    </div>
                </div>

                // 批量翻译卡片
                <div class="rounded-lg shadow-lg p-6 themed-bg-surface0 border border-transparent hover:border-green-200 dark:hover:border-green-700 transition-all cursor-pointer"
                     on:click=move |_| {
                         let navigate = use_navigate();
                         navigate("/batch", Default::default());
                     }>
                    <div class="flex items-center mb-4">
                        <div class="p-2 bg-green-100 dark:bg-green-900 rounded-lg mr-3">
                            <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold themed-text">"批量翻译"</h3>
                            <p class="text-sm themed-subtext">"翻译整个文档网站"</p>
                        </div>
                    </div>
                    <p class="themed-subtext text-sm mb-4">
                        "输入文档网站首页，自动解析目录结构，批量翻译所有页面并打包下载。适合翻译完整的技术文档。"
                    </p>
                    <div class="text-sm text-green-600 dark:text-green-400 font-medium">
                        "→ 点击切换到批量翻译"
                    </div>
                </div>
            </div>
            
            <div class="rounded-lg shadow-lg p-6 themed-bg-surface0">
                <div class="space-y-4">
                    <UrlInput 
                        url=url
                        set_url=set_url
                        on_submit=handle_translate
                        is_loading=translation.is_loading
                    />
                    
                    <ProgressIndicator 
                        is_loading=translation.is_loading
                        progress_message=translation.progress_message
                        status=translation.status
                    />
                </div>
            </div>

            <TranslationResult 
                translation_result=translation.translation_result
                on_download=download_markdown
            />
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