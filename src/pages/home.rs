use leptos::*;
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
            <h1 class="text-3xl font-bold text-center themed-text">
                "URL内容翻译工具"
            </h1>
            
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