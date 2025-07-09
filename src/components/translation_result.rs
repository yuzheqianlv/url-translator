use crate::theme::use_theme_context;
use leptos::*;

#[component]
pub fn TranslationResult(
    translation_result: ReadSignal<String>,
    on_download: impl Fn(web_sys::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    let theme_context = use_theme_context();

    view! {
        <div class="rounded-lg shadow-lg p-6" style=move || theme_context.get().theme.card_style()>
            <div class="flex justify-between items-center mb-4">
                <h2 class="text-xl font-semibold" style=move || theme_context.get().theme.text_style()>
                    "翻译结果"
                </h2>
                <Show when=move || !translation_result.get().is_empty()>
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
                    when=move || !translation_result.get().is_empty()
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
                        <pre class="whitespace-pre-wrap text-sm leading-relaxed p-4 rounded" style=move || theme_context.get().theme.content_bg_style()>
                            {move || translation_result.get()}
                        </pre>
                    </div>
                </Show>
            </div>
        </div>
    }
}
