use crate::theme::{use_theme, ThemeVariant};
use leptos::*;

#[component]
pub fn ThemeSelector() -> impl IntoView {
    let (current_theme, set_theme) = use_theme();

    let handle_theme_change = move |variant: ThemeVariant| {
        set_theme.set(variant);
    };

    view! {
        <div class="space-y-3">
            <h3 class="text-lg font-medium" style="color: var(--color-text)">
                "主题选择"
            </h3>
            <div class="grid grid-cols-2 gap-3">
                <ThemeOption
                    variant=ThemeVariant::Latte
                    current_theme=current_theme
                    on_select=handle_theme_change
                    name="拿铁 (浅色)"
                    colors=("#dc8a78", "#eff1f5", "#4c4f69")
                />
                <ThemeOption
                    variant=ThemeVariant::Frappe
                    current_theme=current_theme
                    on_select=handle_theme_change
                    name="法拉培 (深色)"
                    colors=("#f2d5cf", "#303446", "#c6d0f5")
                />
                <ThemeOption
                    variant=ThemeVariant::Macchiato
                    current_theme=current_theme
                    on_select=handle_theme_change
                    name="玛奇朵 (深色)"
                    colors=("#f4dbd6", "#24273a", "#cad3f5")
                />
                <ThemeOption
                    variant=ThemeVariant::Mocha
                    current_theme=current_theme
                    on_select=handle_theme_change
                    name="摩卡 (极深)"
                    colors=("#f5e0dc", "#1e1e2e", "#cdd6f4")
                />
            </div>
        </div>
    }
}

#[component]
fn ThemeOption(
    variant: ThemeVariant,
    current_theme: ReadSignal<ThemeVariant>,
    on_select: impl Fn(ThemeVariant) + 'static + Copy,
    name: &'static str,
    colors: (&'static str, &'static str, &'static str), // (accent, background, text)
) -> impl IntoView {
    view! {
        <button
            class="p-3 rounded-lg border-2 transition-all"
            class=move || if current_theme.get() == variant { "border-blue-500" } else { "border-gray-300" }
            style=move || {
                let is_active = current_theme.get() == variant;
                let border_color = if is_active { "var(--color-blue)" } else { "var(--color-surface2)" };
                let bg_color = if is_active { "var(--color-surface1)" } else { "var(--color-surface0)" };
                format!("border-color: {}; background-color: {}", border_color, bg_color)
            }
            on:click=move |_| on_select(variant)
        >
            <div class="flex items-center space-x-3">
                <div class="flex space-x-1">
                    <div
                        class="w-4 h-4 rounded-full"
                        style=format!("background-color: {}", colors.0)
                    ></div>
                    <div
                        class="w-4 h-4 rounded-full"
                        style=format!("background-color: {}", colors.1)
                    ></div>
                    <div
                        class="w-4 h-4 rounded-full"
                        style=format!("background-color: {}", colors.2)
                    ></div>
                </div>
                <span class="text-sm font-medium" style="color: var(--color-text)">
                    {name}
                </span>
            </div>
        </button>
    }
}
