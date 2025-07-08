use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, Blob, Url};
use crate::hooks::use_translation::use_translation;
use crate::components::{UrlInput, TranslationResult, ProgressIndicator, PreviewPanel};
use crate::theme::use_theme_context;

#[component]
pub fn HomePage() -> impl IntoView {
    let translation = use_translation();
    let theme_context = use_theme_context();
    let (url, set_url) = create_signal(String::new());
    let (show_preview, set_show_preview) = create_signal(false);
    
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
                <h1 class="text-3xl font-bold mb-4" style=move || theme_context.get().theme.text_style()>
                    "URL内容翻译工具"
                </h1>
                <p class="mb-6" style=move || theme_context.get().theme.subtext_style()>
                    "智能翻译网页内容，支持代码块保护，提供单页和批量翻译模式"
                </p>
            </div>

            // 翻译模式选择卡片
            <div class="grid md:grid-cols-2 gap-6 mb-8">
                // 单页翻译卡片
                <div class="rounded-lg shadow-lg p-6 transition-all hover:shadow-xl" style=move || theme_context.get().theme.card_style()>
                    <div class="flex items-center mb-4">
                        <div class="p-2 rounded-lg mr-3" style=move || theme_context.get().theme.content_bg_style()>
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.info_color())>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold" style=move || theme_context.get().theme.text_style()>"单页翻译"</h3>
                            <p class="text-sm" style=move || theme_context.get().theme.subtext_style()>"翻译单个网页内容"</p>
                        </div>
                    </div>
                    <p class="text-sm mb-4" style=move || theme_context.get().theme.subtext_style()>
                        "输入网页URL，快速翻译单个页面的内容。支持自动提取正文、保护代码块、生成Markdown文件。"
                    </p>
                    <div class="text-sm font-medium" style=move || format!("color: {};", theme_context.get().theme.success_color())>
                        "✓ 当前模式"
                    </div>
                </div>

                // 批量翻译卡片
                <div class="rounded-lg shadow-lg p-6 transition-all hover:shadow-xl cursor-pointer" 
                     style=move || theme_context.get().theme.card_style()
                     on:click=move |_| {
                         let navigate = use_navigate();
                         navigate("/batch", Default::default());
                     }>
                    <div class="flex items-center mb-4">
                        <div class="p-2 rounded-lg mr-3" style=move || theme_context.get().theme.content_bg_style()>
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.success_color())>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold" style=move || theme_context.get().theme.text_style()>"批量翻译"</h3>
                            <p class="text-sm" style=move || theme_context.get().theme.subtext_style()>"翻译整个文档网站"</p>
                        </div>
                    </div>
                    <p class="text-sm mb-4" style=move || theme_context.get().theme.subtext_style()>
                        "输入文档网站首页，自动解析目录结构，批量翻译所有页面并打包下载。适合翻译完整的技术文档。"
                    </p>
                    <div class="text-sm font-medium" style=move || format!("color: {};", theme_context.get().theme.success_color())>
                        "→ 点击切换到批量翻译"
                    </div>
                </div>
            </div>
            
            <div class="rounded-lg shadow-lg p-6" style=move || theme_context.get().theme.card_style()>
                <div class="space-y-4">
                    <UrlInput 
                        url=url
                        set_url=set_url
                        on_submit=handle_translate
                        is_loading=translation.is_loading
                    />
                    
                    // 预览切换按钮
                    <div class="flex items-center gap-4">
                        <button
                            type="button"
                            class="inline-flex items-center px-3 py-2 text-sm font-medium rounded-md border transition-colors"
                            class:bg-blue-100=move || show_preview.get()
                            class:border-blue-300=move || show_preview.get()
                            class:text-blue-800=move || show_preview.get()
                            class:dark:bg-blue-900=move || show_preview.get()
                            class:dark:border-blue-700=move || show_preview.get()
                            class:dark:text-blue-200=move || show_preview.get()
                            class:bg-gray-100=move || !show_preview.get()
                            class:border-gray-300=move || !show_preview.get()
                            class:text-gray-700=move || !show_preview.get()
                            class:dark:bg-gray-700=move || !show_preview.get()
                            class:dark:border-gray-600=move || !show_preview.get()
                            class:dark:text-gray-300=move || !show_preview.get()
                            on:click=move |_| set_show_preview.update(|show| *show = !*show)
                        >
                            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"></path>
                            </svg>
                            {move || if show_preview.get() { "隐藏预览" } else { "显示预览" }}
                        </button>
                        
                        <span class="text-xs text-gray-500 dark:text-gray-400">
                            "预览功能可以在正式翻译前查看前几段的翻译效果"
                        </span>
                    </div>
                    
                    <ProgressIndicator 
                        is_loading=translation.is_loading
                        progress_message=translation.progress_message
                        status=translation.status
                    />
                </div>
            </div>

            // 预览面板
            <PreviewPanel 
                url=url
                show_preview=show_preview
            />

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