//! View mode selector component for switching between original, translated, and bilingual views

use leptos::*;
use serde::{Deserialize, Serialize};

/// Available view modes for file content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Original,
    Translated,
    Bilingual,
}

impl ViewMode {
    pub fn to_string(&self) -> String {
        match self {
            ViewMode::Original => "原文".to_string(),
            ViewMode::Translated => "译文".to_string(),
            ViewMode::Bilingual => "双语".to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            ViewMode::Original => "显示原始内容".to_string(),
            ViewMode::Translated => "显示翻译内容".to_string(),
            ViewMode::Bilingual => "显示原文译文对照".to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ViewMode::Original => "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
            ViewMode::Translated => "M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129",
            ViewMode::Bilingual => "M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2v0a2 2 0 01-2-2V9a2 2 0 00-2-2H8z",
        }
    }

    pub fn color_classes(&self) -> &'static str {
        match self {
            ViewMode::Original => "bg-blue-100 text-blue-800 border-blue-200 hover:bg-blue-200",
            ViewMode::Translated => "bg-green-100 text-green-800 border-green-200 hover:bg-green-200",
            ViewMode::Bilingual => "bg-purple-100 text-purple-800 border-purple-200 hover:bg-purple-200",
        }
    }

    pub fn active_color_classes(&self) -> &'static str {
        match self {
            ViewMode::Original => "bg-blue-500 text-white border-blue-500",
            ViewMode::Translated => "bg-green-500 text-white border-green-500",
            ViewMode::Bilingual => "bg-purple-500 text-white border-purple-500",
        }
    }
}

/// View mode selector component
#[component]
pub fn ViewModeSelector(
    /// Current view mode
    current_mode: ReadSignal<ViewMode>,
    /// Callback when view mode changes
    on_mode_change: Callback<ViewMode>,
    /// Show as tabs (default) or buttons
    #[prop(optional)]
    show_as_tabs: bool,
    /// Show labels (default true)
    #[prop(optional)]
    show_labels: bool,
    /// Compact mode
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let show_labels = if show_labels { true } else { show_labels };
    let modes = vec![ViewMode::Original, ViewMode::Translated, ViewMode::Bilingual];

    let container_class = if show_as_tabs {
        "flex border-b border-gray-200"
    } else if compact {
        "flex space-x-1"
    } else {
        "flex space-x-2"
    };

    view! {
        <div class=container_class>
            <For
                each=move || modes.clone()
                key=|mode| format!("{:?}", mode)
                children=move |mode| {
                    let is_active = create_memo(move |_| current_mode.get() == mode);
                    
                    let button_class = move || {
                        if show_as_tabs {
                            if is_active.get() {
                                "flex items-center px-4 py-2 border-b-2 border-blue-500 text-blue-600 font-medium"
                            } else {
                                "flex items-center px-4 py-2 border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        } else {
                            let base_class = if compact {
                                "flex items-center justify-center px-2 py-1 text-xs border rounded transition-colors"
                            } else {
                                "flex items-center px-3 py-2 text-sm border rounded-lg transition-colors"
                            };
                            
                            if is_active.get() {
                                format!("{} {}", base_class, mode.active_color_classes())
                            } else {
                                format!("{} {}", base_class, mode.color_classes())
                            }
                        }
                    };

                    let on_click = move |_| {
                        on_mode_change.call(mode);
                    };

                    view! {
                        <button
                            class=button_class
                            on:click=on_click
                            title=mode.description()
                        >
                            <svg class=move || if compact { "h-3 w-3" } else { "h-4 w-4" } fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=mode.icon()/>
                            </svg>
                            <Show when=move || show_labels>
                                <span class=move || if compact { "ml-1" } else { "ml-2" }>
                                    {mode.to_string()}
                                </span>
                            </Show>
                        </button>
                    }
                }
            />
        </div>
    }
}

/// Floating view mode selector for overlay on content
#[component]
pub fn FloatingViewModeSelector(
    /// Current view mode
    current_mode: ReadSignal<ViewMode>,
    /// Callback when view mode changes
    on_mode_change: Callback<ViewMode>,
    /// Position from top (default: "1rem")
    #[prop(optional)]
    top: String,
    /// Position from right (default: "1rem")
    #[prop(optional)]
    right: String,
) -> impl IntoView {
    let top = if top.is_empty() { "1rem".to_string() } else { top };
    let right = if right.is_empty() { "1rem".to_string() } else { right };

    let modes = vec![ViewMode::Original, ViewMode::Translated, ViewMode::Bilingual];

    view! {
        <div 
            class="fixed z-40 bg-white rounded-lg shadow-lg border border-gray-200 p-1"
            style:top=top
            style:right=right
        >
            <div class="flex space-x-1">
                <For
                    each=move || modes.clone()
                    key=|mode| format!("{:?}", mode)
                    children=move |mode| {
                        let is_active = create_memo(move |_| current_mode.get() == mode);
                        
                        let button_class = move || {
                            let base_class = "flex items-center justify-center px-2 py-1 text-xs rounded transition-colors";
                            if is_active.get() {
                                format!("{} {}", base_class, mode.active_color_classes())
                            } else {
                                format!("{} {} border", base_class, mode.color_classes())
                            }
                        };

                        let on_click = move |_| {
                            on_mode_change.call(mode);
                        };

                        view! {
                            <button
                                class=button_class
                                on:click=on_click
                                title=mode.description()
                            >
                                <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=mode.icon()/>
                                </svg>
                                <span class="ml-1">{mode.to_string()}</span>
                            </button>
                        }
                    }
                />
            </div>
        </div>
    }
}