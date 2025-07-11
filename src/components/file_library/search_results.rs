//! Search results display component

use leptos::*;
use chrono::{DateTime, Utc};

use crate::types::api_types::{SearchResponse, SearchResult};
use crate::components::file_library::file_preview::FilePreview;

/// Search results display component
#[component]
pub fn SearchResults(
    /// Search results data
    results: ReadSignal<Option<SearchResponse>>,
    /// Callback when a file is selected
    on_file_select: Callback<SearchResult>,
    /// Whether to show loading state
    is_loading: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="w-full max-w-6xl mx-auto mt-6">
            <Show when=is_loading>
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                    <span class="ml-3 text-gray-600">"搜索中..."</span>
                </div>
            </Show>

            <Show when=move || !is_loading.get() && results.get().is_some()>
                {move || {
                    let search_response = results.get().unwrap();
                    view! {
                        <div class="space-y-4">
                            // Search stats
                            <div class="flex items-center justify-between text-sm text-gray-600 px-1">
                                <span>
                                    {format!("找到 {} 个结果", search_response.total)}
                                    {if !search_response.query.is_empty() {
                                        format!(" - 搜索: \"{}\"", search_response.query)
                                    } else {
                                        String::new()
                                    }}
                                </span>
                                <span>
                                    {format!("搜索耗时: {}ms", search_response.search_time_ms)}
                                </span>
                            </div>

                            // Results grid
                            <div class="grid gap-4">
                                <For
                                    each=move || search_response.results.clone()
                                    key=|result| result.translation_id.clone()
                                    children=move |result| {
                                        view! {
                                            <SearchResultItem
                                                result=result.clone()
                                                on_select=on_file_select
                                            />
                                        }
                                    }
                                />
                            </div>

                            // Pagination (if needed)
                            <Show when=move || search_response.total_pages > 1>
                                <div class="flex justify-center mt-8">
                                    <nav class="flex space-x-2">
                                        <For
                                            each=move || (1..=search_response.total_pages).collect::<Vec<_>>()
                                            key=|page| *page
                                            children=move |page| {
                                                let is_current = page == search_response.page;
                                                view! {
                                                    <button
                                                        class=move || format!(
                                                            "px-3 py-1 rounded text-sm {}",
                                                            if is_current {
                                                                "bg-blue-500 text-white"
                                                            } else {
                                                                "bg-gray-200 text-gray-700 hover:bg-gray-300"
                                                            }
                                                        )
                                                        disabled=is_current
                                                    >
                                                        {page}
                                                    </button>
                                                }
                                            }
                                        />
                                    </nav>
                                </div>
                            </Show>
                        </div>
                    }
                }}
            </Show>

            // Empty state
            <Show when=move || !is_loading.get() && results.get().map(|r| r.total == 0).unwrap_or(false)>
                <div class="text-center py-12">
                    <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                    </svg>
                    <h3 class="mt-2 text-lg font-medium text-gray-900">"没有找到匹配的文件"</h3>
                    <p class="mt-1 text-gray-500">"尝试使用不同的关键词搜索"</p>
                </div>
            </Show>
        </div>
    }
}

/// Individual search result item component
#[component]
pub fn SearchResultItem(
    /// Search result data
    result: SearchResult,
    /// Callback when item is selected
    on_select: Callback<SearchResult>,
) -> impl IntoView {
    let result_clone = result.clone();
    
    let on_click = move |_| {
        on_select.call(result_clone.clone());
    };

    let created_at = DateTime::parse_from_rfc3339(&result.created_at)
        .unwrap_or_else(|_| DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap())
        .with_timezone(&Utc);

    view! {
        <div
            class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer bg-white"
            on:click=on_click
        >
            <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                    // File title and URL
                    <div class="flex items-center space-x-2 mb-2">
                        <svg class="h-5 w-5 text-blue-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                        </svg>
                        <h3 class="text-lg font-medium text-gray-900 truncate">
                            {result.title.clone().unwrap_or_else(|| "无标题文档".to_string())}
                        </h3>
                    </div>

                    // URL
                    <p class="text-sm text-blue-600 mb-2 truncate">
                        {result.url.clone()}
                    </p>

                    // Content snippet
                    <p class="text-sm text-gray-600 mb-3 line-clamp-3">
                        {result.content_snippet.clone()}
                    </p>

                    // Metadata
                    <div class="flex items-center space-x-4 text-xs text-gray-500">
                        <span class="flex items-center">
                            <svg class="h-3 w-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                            </svg>
                            {format!("翻译时间: {}", created_at.format("%Y-%m-%d %H:%M"))}
                        </span>
                        <span class="flex items-center">
                            <svg class="h-3 w-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
                            </svg>
                            {format!("大小: ~{}KB", (result.content_snippet.len() * 4) / 1024)}
                        </span>
                        <span class="flex items-center">
                            <svg class="h-3 w-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"/>
                            </svg>
                            {format!("{} → {}", result.source_language, result.target_language)}
                        </span>
                    </div>
                </div>

                // View modes buttons
                <div class="flex flex-col space-y-1 ml-4">
                    <button class="px-3 py-1 text-xs bg-blue-100 text-blue-800 rounded hover:bg-blue-200">
                        "原文"
                    </button>
                    <button class="px-3 py-1 text-xs bg-green-100 text-green-800 rounded hover:bg-green-200">
                        "译文"
                    </button>
                    <button class="px-3 py-1 text-xs bg-purple-100 text-purple-800 rounded hover:bg-purple-200">
                        "双语"
                    </button>
                </div>
            </div>

            // Tags (if any)
            <Show when=move || result.project_name.is_some()>
                <div class="mt-3 flex flex-wrap gap-2">
                    <span class="px-2 py-1 text-xs bg-gray-100 text-gray-800 rounded">
                        {format!("项目: {}", result.project_name.clone().unwrap_or_default())}
                    </span>
                </div>
            </Show>
        </div>
    }
}