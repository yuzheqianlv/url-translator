use crate::theme::{CatppuccinTheme, ThemeVariant};
use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeContext {
    pub current_variant: ThemeVariant,
    pub theme: &'static CatppuccinTheme,
}

impl Default for ThemeContext {
    fn default() -> Self {
        Self {
            current_variant: ThemeVariant::default(),
            theme: CatppuccinTheme::get_theme(&ThemeVariant::default()),
        }
    }
}

#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
    let (theme_variant, set_theme_variant) = create_signal(ThemeVariant::default());

    // Load theme from localStorage
    create_effect(move |_| {
        if let Some(storage) = web_sys::window().unwrap().local_storage().unwrap() {
            if let Ok(Some(stored_theme)) = storage.get_item("theme_variant") {
                if let Ok(variant) = serde_json::from_str::<ThemeVariant>(&stored_theme) {
                    set_theme_variant.set(variant);
                }
            }
        }
    });

    // Apply theme when variant changes
    create_effect(move |_| {
        let variant = theme_variant.get();
        let theme = CatppuccinTheme::get_theme(&variant);
        theme.apply_to_document();

        // Save to localStorage
        if let Some(storage) = web_sys::window().unwrap().local_storage().unwrap() {
            if let Ok(serialized) = serde_json::to_string(&variant) {
                let _ = storage.set_item("theme_variant", &serialized);
            }
        }
    });

    // 创建主题上下文，包含当前主题实例
    let theme_context = create_memo(move |_| {
        let variant = theme_variant.get();
        ThemeContext {
            current_variant: variant,
            theme: CatppuccinTheme::get_theme(&variant),
        }
    });

    provide_context((theme_variant, set_theme_variant));
    provide_context(theme_context);

    children()
}

pub fn use_theme() -> (ReadSignal<ThemeVariant>, WriteSignal<ThemeVariant>) {
    use_context::<(ReadSignal<ThemeVariant>, WriteSignal<ThemeVariant>)>()
        .expect("Theme context not found. Make sure to wrap your app with ThemeProvider.")
}

pub fn use_theme_context() -> Memo<ThemeContext> {
    use_context::<Memo<ThemeContext>>()
        .expect("Theme context not found. Make sure to wrap your app with ThemeProvider.")
}
