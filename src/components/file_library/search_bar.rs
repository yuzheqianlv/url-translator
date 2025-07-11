//! Search bar component with debouncing and real-time search

use leptos::*;
use leptos_use::{use_debounce_fn, use_window_event_listener};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, KeyboardEvent};

use crate::services::api_client::ApiClient;
use crate::types::api_types::{SearchResponse, SearchSuggestionsResponse};

/// Search bar component with debounced real-time search
#[component]
pub fn SearchBar(
    /// Current search query
    query: ReadSignal<String>,
    /// Setter for search query
    set_query: WriteSignal<String>,
    /// Callback when search is triggered
    on_search: Callback<String>,
    /// Callback when suggestions are received
    on_suggestions: Callback<Vec<String>>,
    /// Whether search is loading
    is_loading: ReadSignal<bool>,
    /// Placeholder text
    #[prop(optional)]
    placeholder: String,
) -> impl IntoView {
    let placeholder = if placeholder.is_empty() {
        "搜索所有翻译文件...".to_string()
    } else {
        placeholder
    };

    let (suggestions, set_suggestions) = create_signal(Vec::<String>::new());
    let (show_suggestions, set_show_suggestions) = create_signal(false);
    let (selected_suggestion, set_selected_suggestion) = create_signal(0);
    let input_ref = create_node_ref::<html::Input>();

    // Debounced search function
    let debounced_search = use_debounce_fn(
        move |search_query: String| {
            if search_query.len() >= 2 {
                on_search.call(search_query);
            }
        },
        300.0,
    );

    // Debounced suggestions function
    let debounced_suggestions = use_debounce_fn(
        move |search_query: String| {
            if search_query.len() >= 2 {
                spawn_local(async move {
                    if let Ok(api_client) = ApiClient::new() {
                        if let Ok(response) = api_client.get_search_suggestions(&search_query, 5).await {
                            set_suggestions.set(response.suggestions.clone());
                            set_show_suggestions.set(true);
                            on_suggestions.call(response.suggestions);
                        }
                    }
                });
            } else {
                set_show_suggestions.set(false);
            }
        },
        150.0,
    );

    // Handle input changes
    let on_input = move |ev: Event| {
        let input = ev.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        let value = input.value();
        set_query.set(value.clone());
        set_selected_suggestion.set(0);
        
        if value.is_empty() {
            set_show_suggestions.set(false);
        } else {
            debounced_search.call(value.clone());
            debounced_suggestions.call(value);
        }
    };

    // Handle keyboard navigation
    let on_keydown = move |ev: KeyboardEvent| {
        let key = ev.key();
        match key.as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let current_suggestions = suggestions.get();
                if !current_suggestions.is_empty() {
                    let new_index = (selected_suggestion.get() + 1) % current_suggestions.len();
                    set_selected_suggestion.set(new_index);
                    set_show_suggestions.set(true);
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                let current_suggestions = suggestions.get();
                if !current_suggestions.is_empty() {
                    let new_index = if selected_suggestion.get() == 0 {
                        current_suggestions.len() - 1
                    } else {
                        selected_suggestion.get() - 1
                    };
                    set_selected_suggestion.set(new_index);
                    set_show_suggestions.set(true);
                }
            }
            "Enter" => {
                ev.prevent_default();
                if show_suggestions.get() {
                    let current_suggestions = suggestions.get();
                    if let Some(suggestion) = current_suggestions.get(selected_suggestion.get()) {
                        set_query.set(suggestion.clone());
                        set_show_suggestions.set(false);
                        on_search.call(suggestion.clone());
                    }
                } else {
                    on_search.call(query.get());
                }
            }
            "Escape" => {
                set_show_suggestions.set(false);
                if let Some(input) = input_ref.get() {
                    let _ = input.blur();
                }
            }
            _ => {}
        }
    };

    // Handle suggestion click
    let on_suggestion_click = move |suggestion: String| {
        set_query.set(suggestion.clone());
        set_show_suggestions.set(false);
        on_search.call(suggestion);
    };

    // Handle focus events
    let on_focus = move |_| {
        if !suggestions.get().is_empty() && !query.get().is_empty() {
            set_show_suggestions.set(true);
        }
    };

    // Handle click outside to hide suggestions
    use_window_event_listener("click", move |_| {
        set_show_suggestions.set(false);
    });

    view! {
        <div class="relative w-full max-w-4xl mx-auto">
            <div class="relative">
                // Search icon
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                    </svg>
                </div>

                // Search input
                <input
                    ref=input_ref
                    type="text"
                    class="w-full pl-10 pr-12 py-4 text-lg border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white shadow-sm placeholder-gray-500"
                    placeholder=placeholder
                    value=query
                    on:input=on_input
                    on:keydown=on_keydown
                    on:focus=on_focus
                    on:click=move |ev| ev.stop_propagation()
                />

                // Loading indicator
                <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
                    <Show when=is_loading>
                        <svg class="animate-spin h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                        </svg>
                    </Show>
                </div>

                // Powered by MeiliSearch indicator
                <div class="absolute -bottom-6 right-0 text-xs text-gray-400">
                    "Powered by MeiliSearch"
                </div>
            </div>

            // Search suggestions dropdown
            <Show when=show_suggestions>
                <div class="absolute z-50 w-full mt-1 bg-white border border-gray-300 rounded-lg shadow-lg max-h-96 overflow-y-auto">
                    <For
                        each=move || suggestions.get()
                        key=|suggestion| suggestion.clone()
                        children=move |suggestion| {
                            let is_selected = create_memo(move |_| {
                                suggestions.get().get(selected_suggestion.get()) == Some(&suggestion)
                            });
                            
                            view! {
                                <div
                                    class=move || format!(
                                        "px-4 py-2 cursor-pointer hover:bg-gray-100 {}",
                                        if is_selected.get() { "bg-blue-50" } else { "" }
                                    )
                                    on:click=move |_| on_suggestion_click(suggestion.clone())
                                >
                                    <div class="flex items-center">
                                        <svg class="h-4 w-4 text-gray-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                                        </svg>
                                        <span class="text-sm text-gray-700">{suggestion.clone()}</span>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}

