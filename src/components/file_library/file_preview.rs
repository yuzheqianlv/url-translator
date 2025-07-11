//! File preview card component for search results

use leptos::*;
use chrono::{DateTime, Utc};

use crate::types::api_types::SearchResult;

/// File preview card component for displaying search results
#[component]
pub fn FilePreview(
    /// File data from search results
    file: SearchResult,
    /// Callback when file is clicked
    on_click: Callback<SearchResult>,
    /// Show compact view
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let file_clone = file.clone();
    let on_card_click = move |_| {
        on_click.call(file_clone.clone());
    };

    let created_at = DateTime::parse_from_rfc3339(&file.created_at)
        .unwrap_or_else(|_| DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap())
        .with_timezone(&Utc);

    // Extract domain from URL for display
    let domain = file.url.split('/').nth(2).unwrap_or("unknown").to_string();
    
    // Estimate file size based on content snippet length
    let estimated_size_kb = (file.content_snippet.len() * 4) / 1024;

    let card_class = if compact {
        "border border-gray-200 rounded-lg p-3 hover:shadow-md transition-all duration-200 cursor-pointer bg-white hover:border-blue-300"
    } else {
        "border border-gray-200 rounded-lg p-4 hover:shadow-lg transition-all duration-200 cursor-pointer bg-white hover:border-blue-300"
    };

    view! {
        <div class=card_class on:click=on_card_click>
            <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                    // Header with icon and title
                    <div class="flex items-center space-x-2 mb-2">
                        <svg class="h-5 w-5 text-blue-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                        </svg>
                        <h3 class=move || if compact {
                            "text-base font-medium text-gray-900 truncate"
                        } else {
                            "text-lg font-medium text-gray-900 truncate"
                        }>
                            {file.title.clone().unwrap_or_else(|| extract_title_from_url(&file.url))}
                        </h3>
                    </div>

                    // URL and domain
                    <div class="flex items-center space-x-2 mb-2">
                        <span class="text-xs text-gray-500 bg-gray-100 px-2 py-1 rounded">
                            {domain}
                        </span>
                        <span class="text-xs text-blue-600 truncate">
                            {file.url.clone()}
                        </span>
                    </div>

                    // Content snippet
                    <Show when=move || !compact>
                        <p class="text-sm text-gray-600 mb-3 line-clamp-3">
                            {file.content_snippet.clone()}
                        </p>
                    </Show>

                    // Metadata row
                    <div class=move || if compact {
                        "flex items-center space-x-3 text-xs text-gray-500"
                    } else {
                        "flex items-center space-x-4 text-xs text-gray-500"
                    }>
                        <span class="flex items-center">
                            <svg class="h-3 w-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                            </svg>
                            {if compact {
                                created_at.format("%m-%d").to_string()
                            } else {
                                created_at.format("%Y-%m-%d %H:%M").to_string()
                            }}
                        </span>
                        <span class="flex items-center">
                            <svg class="h-3 w-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
                            </svg>
                            {format!("{}KB", estimated_size_kb.max(1))}
                        </span>
                        <span class="flex items-center">
                            <svg class="h-3 w-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"/>
                            </svg>
                            {format!("{} → {}", file.source_language, file.target_language)}
                        </span>
                    </div>

                    // Project tags
                    <Show when=move || file.project_name.is_some()>
                        <div class="mt-2">
                            <span class="inline-flex items-center px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded">
                                <svg class="h-3 w-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/>
                                </svg>
                                {file.project_name.clone().unwrap()}
                            </span>
                        </div>
                    </Show>
                </div>

                // Action buttons (only in non-compact mode)
                <Show when=move || !compact>
                    <div class="flex flex-col space-y-1 ml-4">
                        <button 
                            class="px-3 py-1 text-xs bg-blue-100 text-blue-800 rounded hover:bg-blue-200 transition-colors"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                // TODO: Handle original view
                            }
                        >
                            "原文"
                        </button>
                        <button 
                            class="px-3 py-1 text-xs bg-green-100 text-green-800 rounded hover:bg-green-200 transition-colors"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                // TODO: Handle translated view
                            }
                        >
                            "译文"
                        </button>
                        <button 
                            class="px-3 py-1 text-xs bg-purple-100 text-purple-800 rounded hover:bg-purple-200 transition-colors"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                // TODO: Handle bilingual view
                            }
                        >
                            "双语"
                        </button>
                    </div>
                </Show>
            </div>

            // Relevance score indicator (if available)
            <Show when=move || file.relevance_score > 0.0>
                <div class="mt-2 flex items-center">
                    <div class="flex-1 bg-gray-200 rounded-full h-1">
                        <div 
                            class="bg-blue-500 h-1 rounded-full"
                            style:width=format!("{}%", (file.relevance_score * 100.0) as i32)
                        ></div>
                    </div>
                    <span class="ml-2 text-xs text-gray-500">
                        {format!("匹配度: {:.0}%", file.relevance_score * 100.0)}
                    </span>
                </div>
            </Show>
        </div>
    }
}

/// Extract a reasonable title from URL if no title is provided
fn extract_title_from_url(url: &str) -> String {
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(domain) = parsed_url.domain() {
            let path = parsed_url.path();
            if let Some(last_segment) = path.split('/').last() {
                if !last_segment.is_empty() && last_segment != "index.html" {
                    let name = last_segment.split('.').next().unwrap_or(last_segment);
                    if !name.is_empty() {
                        return format!("{} - {}", name, domain);
                    }
                }
            }
            return domain.to_string();
        }
    }
    "无标题文档".to_string()
}