//! Homepage - File Library Search Center
//! 
//! Transformed from a translation-focused interface to a search-centered file management system.
//! This page now serves as the primary interface for searching and managing translated documents.

use leptos::*;
use leptos_router::*;

use crate::components::{
    AuthRequired, 
    file_library::{SearchBar, SearchResults, FileViewer},
};
use crate::hooks::use_auth::{use_auth, AuthStatus};
use crate::services::api_client::{ApiClient, SearchRequest};
use crate::types::api_types::{SearchResponse, SearchResult};
use crate::theme::use_theme_context;

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let theme_context = use_theme_context();
    
    // Search state
    let (search_query, set_search_query) = create_signal(String::new());
    let (search_results, set_search_results) = create_signal::<Option<SearchResponse>>(None);
    let (is_searching, set_is_searching) = create_signal(false);
    let (search_suggestions, set_search_suggestions) = create_signal(Vec::<String>::new());
    
    // File viewer state
    let (selected_file, set_selected_file) = create_signal::<Option<SearchResult>>(None);
    let (is_viewer_open, set_is_viewer_open) = create_signal(false);
    
    // Recent files state (shown when no search is active)
    let (recent_files, set_recent_files) = create_signal::<Option<SearchResponse>>(None);
    let (is_loading_recent, set_is_loading_recent) = create_signal(false);

    // Load recent files on component mount
    create_effect(move |_| {
        if let AuthStatus::Authenticated(_) = auth.auth_status.get() {
            set_is_loading_recent.set(true);
            spawn_local(async move {
                if let Ok(api_client) = ApiClient::new() {
                    let request = SearchRequest {
                        query: "".to_string(), // Empty query to get recent files
                        page: Some(1),
                        per_page: Some(10),
                        project_id: None,
                        source_language: None,
                        target_language: None,
                    };
                    
                    match api_client.search_translations(request).await {
                        Ok(response) => {
                            set_recent_files.set(Some(response));
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("Failed to load recent files: {}", e).into());
                        }
                    }
                }
                set_is_loading_recent.set(false);
            });
        }
    });

    // Handle search
    let handle_search = move |query: String| {
        if query.trim().is_empty() {
            set_search_results.set(None);
            return;
        }

        set_is_searching.set(true);
        let query_clone = query.clone();
        set_search_query.set(query);
        
        spawn_local(async move {
            if let Ok(api_client) = ApiClient::new() {
                let request = SearchRequest {
                    query: query_clone,
                    page: Some(1),
                    per_page: Some(20),
                    project_id: None,
                    source_language: None,
                    target_language: None,
                };
                
                match api_client.search_translations(request).await {
                    Ok(response) => {
                        set_search_results.set(Some(response));
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Search failed: {}", e).into());
                        set_search_results.set(Some(SearchResponse {
                            results: vec![],
                            total: 0,
                            page: 1,
                            per_page: 20,
                            total_pages: 0,
                            query: query_clone,
                            search_time_ms: 0,
                            suggestions: vec![],
                        }));
                    }
                }
            }
            set_is_searching.set(false);
        });
    };

    // Handle search suggestions
    let handle_suggestions = move |suggestions: Vec<String>| {
        set_search_suggestions.set(suggestions);
    };

    // Handle file selection
    let handle_file_select = move |file: SearchResult| {
        set_selected_file.set(Some(file));
        set_is_viewer_open.set(true);
    };

    // Handle viewer close
    let handle_viewer_close = Callback::new(move |_: ()| {
        set_is_viewer_open.set(false);
        set_selected_file.set(None);
    });

    // Quick translation entry (floating action button)
    let navigate_to_translation = move |_| {
        let navigate = use_navigate();
        navigate("/translate", Default::default());
    };

    view! {
        <div class="min-h-screen" style=move || theme_context.get().theme.bg_style()>
            // Header section with search
            <div class="w-full bg-white shadow-sm border-b border-gray-200">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
                    // Title and search bar
                    <div class="text-center mb-6">
                        <h1 class="text-3xl font-bold text-gray-900 mb-2">
                            "🔍 文件库搜索"
                        </h1>
                        <p class="text-gray-600 mb-6">
                            "搜索和管理您的翻译文件"
                        </p>
                        
                        <AuthRequired message="请先登录后搜索您的翻译文件".to_string()>
                            <SearchBar
                                query=search_query
                                set_query=set_search_query
                                on_search=Callback::new(handle_search)
                                on_suggestions=Callback::new(handle_suggestions)
                                is_loading=is_searching
                                placeholder="搜索所有翻译文件...".to_string()
                            />
                        </AuthRequired>
                    </div>
                </div>
            </div>

            // Main content area
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                <AuthRequired message="登录后即可搜索和管理您的翻译历史".to_string()>
                    // Search results or recent files
                    <Show
                        when=move || search_results.get().is_some()
                        fallback=move || view! {
                            // Recent files section
                            <div class="space-y-6">
                                <div class="flex items-center justify-between">
                                    <h2 class="text-xl font-semibold text-gray-900">
                                        "📁 最近翻译的文件"
                                    </h2>
                                    <div class="text-sm text-gray-500">
                                        "显示最近10个翻译文件"
                                    </div>
                                </div>
                                
                                <Show when=is_loading_recent>
                                    <div class="flex justify-center items-center py-12">
                                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                                        <span class="ml-3 text-gray-600">"加载中..."</span>
                                    </div>
                                </Show>

                                <Show when=move || !is_loading_recent.get() && recent_files.get().is_some()>
                                    <SearchResults
                                        results=recent_files
                                        on_file_select=Callback::new(handle_file_select)
                                        is_loading=create_signal(false).0
                                    />
                                </Show>

                                <Show when=move || !is_loading_recent.get() && recent_files.get().map(|r| r.total == 0).unwrap_or(false)>
                                    <div class="text-center py-12">
                                        <div class="mx-auto h-12 w-12 text-gray-400 mb-4">
                                            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                            </svg>
                                        </div>
                                        <h3 class="text-lg font-medium text-gray-900 mb-2">"暂无翻译文件"</h3>
                                        <p class="text-gray-500 mb-4">"开始您的第一次翻译吧"</p>
                                        <button
                                            class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                                            on:click=navigate_to_translation
                                        >
                                            <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                                            </svg>
                                            "开始翻译"
                                        </button>
                                    </div>
                                </Show>
                            </div>
                        }
                    >
                        // Search results
                        <SearchResults
                            results=search_results
                            on_file_select=Callback::new(handle_file_select)
                            is_loading=is_searching
                        />
                    </Show>
                </AuthRequired>
            </div>

            // Quick translation floating action
            <div class="fixed bottom-6 right-6">
                <button
                    class="bg-blue-600 hover:bg-blue-700 text-white rounded-full p-4 shadow-lg transition-all duration-200 hover:scale-105"
                    on:click=navigate_to_translation
                    title="快速翻译"
                >
                    <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"/>
                    </svg>
                </button>
            </div>

            // Right corner menu tooltip
            <div class="fixed top-4 right-4 z-40">
                <div class="bg-blue-50 border border-blue-200 rounded-lg p-3 text-sm text-blue-800 shadow-sm">
                    <div class="flex items-center">
                        <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                        </svg>
                        <span>"点击右上角菜单访问其他功能"</span>
                    </div>
                </div>
            </div>

            // File viewer modal
            <FileViewer
                is_open=is_viewer_open
                on_close=handle_viewer_close
                file=selected_file
            />
        </div>
    }
}