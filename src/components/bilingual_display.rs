use crate::theme::use_theme_context;
use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayMode {
    /// 仅显示翻译结果
    TranslationOnly,
    /// 仅显示原文
    OriginalOnly,
    /// 双语对照模式：译文在前，原文在后
    Bilingual,
}

#[component]
pub fn BilingualDisplay(
    original_content: ReadSignal<String>,
    translated_content: ReadSignal<String>,
    display_mode: ReadSignal<DisplayMode>,
    on_download: impl Fn(web_sys::MouseEvent) + 'static + Copy,
    on_mode_change: impl Fn(DisplayMode) + 'static + Copy,
) -> impl IntoView {
    let theme_context = use_theme_context();

    view! {
        <div class="rounded-lg shadow-lg p-6" style=move || theme_context.get().theme.card_style()>
            <div class="flex justify-between items-center mb-4">
                <div class="flex items-center space-x-4">
                    <h2 class="text-xl font-semibold" style=move || theme_context.get().theme.text_style()>
                        "翻译结果"
                    </h2>
                    
                    // 模式切换按钮组
                    <div class="flex rounded-md overflow-hidden border" style=move || theme_context.get().theme.border_style()>
                        <button
                            class="px-3 py-1 text-sm transition-colors"
                            class:font-medium=move || display_mode.get() == DisplayMode::TranslationOnly
                            style=move || {
                                if display_mode.get() == DisplayMode::TranslationOnly {
                                    theme_context.get().theme.button_primary_style()
                                } else {
                                    theme_context.get().theme.button_secondary_style()
                                }
                            }
                            on:click=move |_| on_mode_change(DisplayMode::TranslationOnly)
                        >
                            "译文"
                        </button>
                        <button
                            class="px-3 py-1 text-sm transition-colors border-l border-r"
                            class:font-medium=move || display_mode.get() == DisplayMode::Bilingual
                            style=move || {
                                let theme = theme_context.get().theme;
                                let base_style = if display_mode.get() == DisplayMode::Bilingual {
                                    theme.button_primary_style()
                                } else {
                                    theme.button_secondary_style()
                                };
                                format!("{};{}", base_style, theme.border_style())
                            }
                            on:click=move |_| on_mode_change(DisplayMode::Bilingual)
                        >
                            "双语"
                        </button>
                        <button
                            class="px-3 py-1 text-sm transition-colors"
                            class:font-medium=move || display_mode.get() == DisplayMode::OriginalOnly
                            style=move || {
                                if display_mode.get() == DisplayMode::OriginalOnly {
                                    theme_context.get().theme.button_primary_style()
                                } else {
                                    theme_context.get().theme.button_secondary_style()
                                }
                            }
                            on:click=move |_| on_mode_change(DisplayMode::OriginalOnly)
                        >
                            "原文"
                        </button>
                    </div>
                </div>

                <Show when=move || !translated_content.get().is_empty()>
                    <button
                        class="px-4 py-2 rounded-md flex items-center space-x-2 transition-colors hover:opacity-90"
                        style=move || theme_context.get().theme.button_success_style()
                        on:click=on_download
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
                    when=move || !translated_content.get().is_empty()
                    fallback=move || {
                        let theme = theme_context.get().theme;
                        view! {
                            <div class="flex items-center justify-center h-48" style=theme.subtext_style()>
                                <div class="text-center">
                                    <svg class="w-12 h-12 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=theme.muted_text_style()>
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                    </svg>
                                    <p class="text-lg font-medium">"暂无翻译内容"</p>
                                    <p class="text-sm">"请输入URL并点击翻译按钮"</p>
                                </div>
                            </div>
                        }
                    }
                >
                    <div class="prose prose-sm max-w-none">
                        {move || match display_mode.get() {
                            DisplayMode::TranslationOnly => {
                                view! {
                                    <pre class="whitespace-pre-wrap text-sm leading-relaxed p-4 rounded" style=move || theme_context.get().theme.content_bg_style()>
                                        {move || translated_content.get()}
                                    </pre>
                                }.into_view()
                            }
                            DisplayMode::OriginalOnly => {
                                view! {
                                    <pre class="whitespace-pre-wrap text-sm leading-relaxed p-4 rounded" style=move || theme_context.get().theme.content_bg_style()>
                                        {move || original_content.get()}
                                    </pre>
                                }.into_view()
                            }
                            DisplayMode::Bilingual => {
                                view! {
                                    <BilingualContent 
                                        original_content=original_content
                                        translated_content=translated_content
                                    />
                                }.into_view()
                            }
                        }}
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn BilingualContent(
    original_content: ReadSignal<String>,
    translated_content: ReadSignal<String>,
) -> impl IntoView {
    let theme_context = use_theme_context();

    // 创建双语对照内容的计算信号
    let bilingual_lines = create_memo(move |_| {
        let original = original_content.get();
        let translated = translated_content.get();
        
        if original.is_empty() || translated.is_empty() {
            return Vec::new();
        }

        create_bilingual_pairs(&original, &translated)
    });

    view! {
        <div class="space-y-4">
            <For
                each=move || bilingual_lines.get()
                key=|item| item.0.clone()
                children=move |line_pair| {
                    let theme = theme_context.get().theme;
                    view! {
                        <div class="border-l-4 pl-4 space-y-2" style=format!("border-left-color: {}", theme.accent_color())>
                            // 译文行（在前）
                            <div class="relative">
                                <div class="absolute -left-6 top-0 w-4 h-4 rounded-full text-xs flex items-center justify-center text-white font-bold" style=format!("background-color: {}", theme.accent_color())>
                                    "译"
                                </div>
                                <pre class="whitespace-pre-wrap text-sm leading-relaxed p-3" style=theme.translated_bg_style()>
                                    {line_pair.1.clone()}
                                </pre>
                            </div>
                            
                            // 原文行（在后）
                            <div class="relative">
                                <div class="absolute -left-6 top-0 w-4 h-4 rounded-full text-xs flex items-center justify-center text-white font-bold" style=format!("background-color: {}", theme.muted_color())>
                                    "原"
                                </div>
                                <pre class="whitespace-pre-wrap text-sm leading-relaxed p-3" style=theme.original_bg_style()>
                                    {line_pair.0.clone()}
                                </pre>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// 创建双语对照的行对
pub fn create_bilingual_pairs(original: &str, translated: &str) -> Vec<(String, String)> {
    let original_lines: Vec<&str> = original.lines().collect();
    let translated_lines: Vec<&str> = translated.lines().collect();
    
    let mut pairs = Vec::new();
    let max_len = original_lines.len().max(translated_lines.len());
    
    for i in 0..max_len {
        let original_line = original_lines.get(i).unwrap_or(&"").to_string();
        let translated_line = translated_lines.get(i).unwrap_or(&"").to_string();
        
        // 跳过空行对
        if original_line.trim().is_empty() && translated_line.trim().is_empty() {
            continue;
        }
        
        pairs.push((original_line, translated_line));
    }
    
    pairs
}

/// 扩展主题以支持双语模式样式
pub trait BilingualThemeExt {
    fn translated_bg_style(&self) -> String;
    fn original_bg_style(&self) -> String;
    fn accent_color(&self) -> String;
    fn muted_color(&self) -> String;
    fn border_style(&self) -> String;
}

impl BilingualThemeExt for crate::theme::CatppuccinTheme {
    fn translated_bg_style(&self) -> String {
        format!(
            "background-color: {}; color: {}; border: 1px solid {}; border-radius: 6px;",
            self.surface1, self.text, self.blue
        )
    }

    fn original_bg_style(&self) -> String {
        format!(
            "background-color: {}; color: {}; border: 1px solid {}; border-radius: 6px;",
            self.surface0, self.subtext1, self.overlay0
        )
    }

    fn accent_color(&self) -> String {
        self.blue.to_string()
    }

    fn muted_color(&self) -> String {
        self.overlay0.to_string()
    }

    fn border_style(&self) -> String {
        format!("border-color: {};", self.surface2)
    }
}