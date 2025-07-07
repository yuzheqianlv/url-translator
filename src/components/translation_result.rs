use leptos::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, Blob, Url};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[component]
pub fn TranslationResult() -> impl IntoView {
    let translation_result = use_context::<ReadSignal<String>>()
        .expect("Translation result context not found");
    
    let download_markdown = move |_| {
        let content = translation_result.get();
        if content.is_empty() {
            return;
        }
        
        let _ = create_and_download_file(&content, "translated_content.md", "text/markdown");
    };
    
    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            <div class="flex justify-between items-center mb-4">
                <h2 class="text-xl font-semibold text-gray-800">
                    "翻译结果"
                </h2>
                <Show when=move || !translation_result.get().is_empty()>
                    <button
                        class="bg-green-600 text-white px-4 py-2 rounded-md hover:bg-green-700 flex items-center space-x-2"
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
                        <pre class="whitespace-pre-wrap text-sm text-gray-800 leading-relaxed">
                            {move || translation_result.get()}
                        </pre>
                    </div>
                </Show>
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