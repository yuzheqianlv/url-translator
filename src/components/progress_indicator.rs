use crate::hooks::use_translation::TranslationStatus;
use leptos::*;

#[component]
pub fn ProgressIndicator(
    is_loading: ReadSignal<bool>,
    progress_message: ReadSignal<String>,
    status: ReadSignal<TranslationStatus>,
) -> impl IntoView {
    view! {
        <Show when=move || is_loading.get() && !progress_message.get().is_empty()>
            <div class="border px-4 py-3 rounded flex items-center themed-alert-info">
                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 themed-progress" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                <div class="flex-1">
                    <div class="font-medium">
                        {progress_message}
                    </div>
                    <div class="text-sm mt-1">
                        {move || {
                            match status.get() {
                                TranslationStatus::ExtractingContent => "正在从网页提取内容...",
                                TranslationStatus::Translating => "正在进行翻译处理...",
                                TranslationStatus::Completed => "处理完成！",
                                TranslationStatus::Failed(_) => "处理失败",
                                TranslationStatus::Idle => "",
                            }
                        }}
                    </div>
                </div>
            </div>
        </Show>
    }
}
