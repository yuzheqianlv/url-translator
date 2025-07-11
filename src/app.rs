use crate::error::{ErrorDisplay, ErrorProvider};
use crate::pages::{BatchPage, HistoryPage, HomePage, ProjectsPage, SettingsPage, TranslationPage};
use crate::components::{Header, NotificationContainer};
use crate::theme::{use_theme_context, ThemeProvider};
use crate::hooks::AsyncTranslationProvider;
use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <ErrorProvider>
            <ThemeProvider>
                <AsyncTranslationProvider>
                    <AppContent />
                </AsyncTranslationProvider>
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
                <Header />
                <main class="container mx-auto px-4 py-8">
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/translate" view=TranslationPage/>
                        <Route path="/batch" view=BatchPage/>
                        <Route path="/settings" view=SettingsPage/>
                        <Route path="/history" view=HistoryPage/>
                        <Route path="/projects" view=ProjectsPage/>
                    </Routes>
                </main>
            </Router>
            <ErrorDisplay />
            <NotificationContainer />
        </div>
    }
}

