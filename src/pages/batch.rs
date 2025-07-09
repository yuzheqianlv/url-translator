use crate::components::BatchTranslation;
use leptos::*;
use leptos_router::*;

#[component]
pub fn BatchPage() -> impl IntoView {
    view! {
        <div class="max-w-6xl mx-auto">
            <div class="mb-8">
                <div class="flex items-center justify-between mb-4">
                    <div>
                        <h1 class="text-3xl font-bold mb-2 themed-text">
                            "批量翻译文档网站"
                        </h1>
                        <p class="themed-subtext">
                            "自动提取文档网站的目录结构，批量翻译所有页面并打包下载"
                        </p>
                    </div>
                    <A href="/"
                       class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-blue-600 bg-blue-100 hover:bg-blue-200 dark:text-blue-400 dark:bg-blue-900 dark:hover:bg-blue-800 transition-colors">
                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                        </svg>
                        "单页翻译"
                    </A>
                </div>

                // 功能说明卡片
                <div class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-4 mb-6">
                    <div class="flex">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-amber-400" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path>
                            </svg>
                        </div>
                        <div class="ml-3">
                            <h3 class="text-sm font-medium text-amber-800 dark:text-amber-200">
                                "使用提示"
                            </h3>
                            <div class="mt-2 text-sm text-amber-700 dark:text-amber-300">
                                <ul class="list-disc list-inside space-y-1">
                                    <li>"输入文档网站的首页URL（如: https://rust-lang.github.io/mdBook/）"</li>
                                    <li>"系统会自动解析目录结构并提取所有文档链接"</li>
                                    <li>"翻译过程中会保护代码块不被翻译"</li>
                                    <li>"完成后会自动打包所有翻译文件并下载"</li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <BatchTranslation />
        </div>
    }
}
