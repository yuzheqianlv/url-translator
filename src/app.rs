use leptos::*;
use leptos_router::*;
use crate::theme::{ThemeProvider, use_theme, use_theme_context, ThemeVariant};
use crate::error::{ErrorProvider, ErrorDisplay};
use crate::pages::{HomePage, SettingsPage, HistoryPage, BatchPage};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <ErrorProvider>
            <ThemeProvider>
                <AppContent />
            </ThemeProvider>
        </ErrorProvider>
    }
}

#[component]
fn AppContent() -> impl IntoView {
    let theme_context = use_theme_context();
    
    view! {
        <div class="min-h-screen" style=move || theme_context.get().theme.base_bg_style()>
            <Router>
                <AppHeader />
                <main class="container mx-auto px-4 py-8">
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/batch" view=BatchPage/>
                        <Route path="/settings" view=SettingsPage/>
                        <Route path="/history" view=HistoryPage/>
                    </Routes>
                </main>
            </Router>
            <ErrorDisplay />
        </div>
    }
}

#[component]
fn AppHeader() -> impl IntoView {
    let (theme_variant, set_theme_variant) = use_theme();
    let theme_context = use_theme_context();
    
    let cycle_theme = move |_| {
        let current = theme_variant.get();
        let next = match current {
            ThemeVariant::Latte => ThemeVariant::Frappe,
            ThemeVariant::Frappe => ThemeVariant::Macchiato,
            ThemeVariant::Macchiato => ThemeVariant::Mocha,
            ThemeVariant::Mocha => ThemeVariant::Latte,
        };
        set_theme_variant.set(next);
    };
    
    view! {
        <header class="shadow-sm" style=move || theme_context.get().theme.nav_style()>
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center space-x-4">
                        <a href="/" class="text-xl font-bold transition-colors hover:opacity-80" style=move || theme_context.get().theme.text_style()>
                            "URL翻译工具"
                        </a>
                    </div>
                    
                    <nav class="flex items-center space-x-6">
                        <a 
                            href="/" 
                            class="transition-colors px-3 py-2 rounded-md hover:opacity-80"
                            style=move || theme_context.get().theme.subtext_style()
                        >
                            "单页翻译"
                        </a>
                        <a 
                            href="/batch" 
                            class="transition-colors px-3 py-2 rounded-md hover:opacity-80"
                            style=move || theme_context.get().theme.subtext_style()
                        >
                            "批量翻译"
                        </a>
                        <a 
                            href="/history" 
                            class="transition-colors px-3 py-2 rounded-md hover:opacity-80"
                            style=move || theme_context.get().theme.subtext_style()
                        >
                            "历史"
                        </a>
                        <a 
                            href="/settings" 
                            class="transition-colors px-3 py-2 rounded-md hover:opacity-80"
                            style=move || theme_context.get().theme.subtext_style()
                        >
                            "设置"
                        </a>
                        <button
                            class="p-2 rounded-md transition-colors hover:opacity-80"
                            style=move || theme_context.get().theme.button_secondary_style()
                            on:click=cycle_theme
                            title=move || format!("当前主题: {}", theme_variant.get())
                        >
                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd" />
                            </svg>
                        </button>
                    </nav>
                </div>
            </div>
        </header>
    }
}