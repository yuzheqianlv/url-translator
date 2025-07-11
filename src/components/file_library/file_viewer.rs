//! Full-screen file viewer component with multiple view modes

use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{window, KeyboardEvent};

use crate::types::api_types::{SearchResult, TranslationResponse};
use crate::components::file_library::view_mode::{ViewMode, ViewModeSelector, FloatingViewModeSelector};
use crate::components::file_library::download::DownloadButton;
use crate::services::api_client::ApiClient;

/// Full-screen file viewer with view mode switching
#[component]
pub fn FileViewer(
    /// Whether the viewer is open
    is_open: ReadSignal<bool>,
    /// Callback to close the viewer
    on_close: Callback<()>,
    /// Search result data for the file
    file: ReadSignal<Option<SearchResult>>,
) -> impl IntoView {
    let (view_mode, set_view_mode) = create_signal(ViewMode::Bilingual);
    let (translation_data, set_translation_data) = create_signal::<Option<TranslationResponse>>(None);
    let (is_loading, set_is_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal::<Option<String>>(None);

    // Load full translation data when file changes
    create_effect(move |_| {
        if let Some(file_data) = file.get() {
            set_is_loading.set(true);
            set_error_message.set(None);
            
            spawn_local(async move {
                match ApiClient::new() {
                    Ok(api_client) => {
                        let translation_id = match uuid::Uuid::parse_str(&file_data.translation_id) {
                            Ok(id) => id,
                            Err(_) => {
                                set_error_message.set(Some("Invalid translation ID".to_string()));
                                set_is_loading.set(false);
                                return;
                            }
                        };

                        match api_client.get_translation(translation_id).await {
                            Ok(translation) => {
                                set_translation_data.set(Some(translation));
                                set_error_message.set(None);
                            }
                            Err(e) => {
                                set_error_message.set(Some(format!("Failed to load translation: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        set_error_message.set(Some(format!("Failed to create API client: {}", e)));
                    }
                }
                set_is_loading.set(false);
            });
        }
    });

    // Handle keyboard shortcuts
    let handle_keydown = move |ev: KeyboardEvent| {
        if !is_open.get() {
            return;
        }

        match ev.key().as_str() {
            "Escape" => {
                on_close.call(());
            }
            "1" => {
                set_view_mode.set(ViewMode::Original);
            }
            "2" => {
                set_view_mode.set(ViewMode::Translated);
            }
            "3" => {
                set_view_mode.set(ViewMode::Bilingual);
            }
            _ => {}
        }
    };

    // Set up keyboard event listener
    create_effect(move |_| {
        if is_open.get() {
            if let Some(window) = window() {
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(handle_keydown) as Box<dyn Fn(KeyboardEvent)>);
                let _ = window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    });

    view! {
        <Show when=is_open>
            <div class="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center p-4">
                <div class="bg-white rounded-lg shadow-xl w-full h-full max-w-7xl max-h-full flex flex-col">
                    // Header
                    <div class="flex items-center justify-between p-4 border-b border-gray-200">
                        <div class="flex items-center space-x-4">
                            <h2 class="text-xl font-semibold text-gray-900">
                                {move || {
                                    file.get().and_then(|f| f.title.clone())
                                        .unwrap_or_else(|| "文件查看器".to_string())
                                }}
                            </h2>
                            <ViewModeSelector
                                current_mode=view_mode
                                on_mode_change=Callback::new(move |mode| set_view_mode.set(mode))
                                show_as_tabs=true
                            />
                        </div>
                        
                        <div class="flex items-center space-x-2">
                            // Download button
                            <Show when=move || translation_data.get().is_some()>
                                <DownloadButton
                                    translation=translation_data
                                    view_mode=view_mode
                                    file_name=move || {
                                        file.get().and_then(|f| f.title.clone())
                                            .unwrap_or_else(|| "translation".to_string())
                                    }
                                />
                            </Show>
                            
                            // Close button
                            <button
                                class="p-2 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-gray-100"
                                on:click=move |_| on_close.call(())
                                title="关闭 (Esc)"
                            >
                                <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                </svg>
                            </button>
                        </div>
                    </div>

                    // Content area
                    <div class="flex-1 overflow-hidden">
                        <Show when=is_loading>
                            <div class="flex items-center justify-center h-full">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                                <span class="ml-3 text-gray-600">"加载中..."</span>
                            </div>
                        </Show>

                        <Show when=move || error_message.get().is_some()>
                            <div class="flex items-center justify-center h-full">
                                <div class="text-center">
                                    <svg class="mx-auto h-12 w-12 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16c-.77.833.192 2.5 1.732 2.5z"/>
                                    </svg>
                                    <h3 class="mt-2 text-lg font-medium text-gray-900">"加载失败"</h3>
                                    <p class="mt-1 text-gray-500">{error_message.get().unwrap_or_default()}</p>
                                </div>
                            </div>
                        </Show>

                        <Show when=move || !is_loading.get() && error_message.get().is_none() && translation_data.get().is_some()>
                            <div class="h-full overflow-auto p-6">
                                <FileContent
                                    translation=translation_data
                                    view_mode=view_mode
                                />
                            </div>
                        </Show>
                    </div>

                    // Footer with keyboard shortcuts
                    <div class="border-t border-gray-200 px-4 py-2 bg-gray-50">
                        <div class="flex items-center justify-between text-sm text-gray-600">
                            <div class="flex items-center space-x-4">
                                <span>"快捷键:"</span>
                                <span>"Esc - 关闭"</span>
                                <span>"1 - 原文"</span>
                                <span>"2 - 译文"</span>
                                <span>"3 - 双语"</span>
                            </div>
                            <div>
                                {move || {
                                    if let Some(file_data) = file.get() {
                                        format!("语言: {} → {}", file_data.source_language, file_data.target_language)
                                    } else {
                                        String::new()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// File content display component based on view mode
#[component]
pub fn FileContent(
    /// Translation data
    translation: ReadSignal<Option<TranslationResponse>>,
    /// Current view mode
    view_mode: ReadSignal<ViewMode>,
) -> impl IntoView {
    view! {
        <div class="max-w-none">
            {move || {
                match (translation.get(), view_mode.get()) {
                    (Some(trans), ViewMode::Original) => {
                        view! {
                            <div class="prose prose-lg max-w-none">
                                <div class="bg-blue-50 border-l-4 border-blue-400 p-4 mb-6">
                                    <h3 class="text-lg font-medium text-blue-900 mb-2">"原始内容"</h3>
                                    <p class="text-blue-700">"以下是从网页提取的原始内容"</p>
                                </div>
                                <div inner_html=format_content(&trans.original_content)></div>
                            </div>
                        }.into_view()
                    }
                    (Some(trans), ViewMode::Translated) => {
                        view! {
                            <div class="prose prose-lg max-w-none">
                                <div class="bg-green-50 border-l-4 border-green-400 p-4 mb-6">
                                    <h3 class="text-lg font-medium text-green-900 mb-2">"翻译内容"</h3>
                                    <p class="text-green-700">"以下是翻译后的内容"</p>
                                </div>
                                <div inner_html=format_content(&trans.translated_content)></div>
                            </div>
                        }.into_view()
                    }
                    (Some(trans), ViewMode::Bilingual) => {
                        view! {
                            <div class="space-y-6">
                                <div class="bg-purple-50 border-l-4 border-purple-400 p-4">
                                    <h3 class="text-lg font-medium text-purple-900 mb-2">"双语对照"</h3>
                                    <p class="text-purple-700">"原文和译文段落级对照显示"</p>
                                </div>
                                <BilingualContent
                                    original_content=trans.original_content.clone()
                                    translated_content=trans.translated_content.clone()
                                />
                            </div>
                        }.into_view()
                    }
                    _ => {
                        view! {
                            <div class="text-center py-12">
                                <p class="text-gray-500">"暂无内容"</p>
                            </div>
                        }.into_view()
                    }
                }
            }}
        </div>
    }
}

/// Bilingual content display with paragraph-level alignment
#[component]
pub fn BilingualContent(
    /// Original content
    original_content: String,
    /// Translated content
    translated_content: String,
) -> impl IntoView {
    // Split content into paragraphs
    let original_paragraphs: Vec<String> = original_content
        .split("\n\n")
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    
    let translated_paragraphs: Vec<String> = translated_content
        .split("\n\n")
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();

    let max_paragraphs = original_paragraphs.len().max(translated_paragraphs.len());

    view! {
        <div class="space-y-4">
            <For
                each=move || (0..max_paragraphs).collect::<Vec<_>>()
                key=|i| *i
                children=move |i| {
                    let original_text = original_paragraphs.get(i).cloned().unwrap_or_default();
                    let translated_text = translated_paragraphs.get(i).cloned().unwrap_or_default();
                    
                    view! {
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 p-4 border border-gray-200 rounded-lg">
                            <div class="space-y-2">
                                <div class="text-xs font-medium text-blue-600 uppercase tracking-wide">"原文"</div>
                                <div class="prose prose-sm text-gray-800" inner_html=format_content(&original_text)></div>
                            </div>
                            <div class="space-y-2">
                                <div class="text-xs font-medium text-green-600 uppercase tracking-wide">"译文"</div>
                                <div class="prose prose-sm text-gray-800" inner_html=format_content(&translated_text)></div>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// Format content for HTML display (basic markdown-like formatting)
fn format_content(content: &str) -> String {
    if content.is_empty() {
        return "<p class=\"text-gray-400 italic\">此段落无内容</p>".to_string();
    }

    // Basic markdown-like formatting
    let mut formatted = content
        // Convert markdown headers
        .replace("### ", "<h3>")
        .replace("## ", "<h2>")
        .replace("# ", "<h1>")
        // Convert bold text
        .replace("**", "<strong>")
        // Convert italic text  
        .replace("*", "<em>")
        // Convert line breaks
        .replace("\n", "<br>");

    // Close header tags
    if formatted.contains("<h1>") {
        formatted = formatted.replace("<h1>", "<h1>").replace("\n", "</h1>\n");
    }
    if formatted.contains("<h2>") {
        formatted = formatted.replace("<h2>", "<h2>").replace("\n", "</h2>\n");
    }
    if formatted.contains("<h3>") {
        formatted = formatted.replace("<h3>", "<h3>").replace("\n", "</h3>\n");
    }

    // Wrap in paragraph if no block elements
    if !formatted.contains("<h1>") && !formatted.contains("<h2>") && !formatted.contains("<h3>") {
        formatted = format!("<p>{}</p>", formatted);
    }

    formatted
}