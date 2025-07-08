use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Copy)]
pub enum ThemeVariant {
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Latte
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeVariant::Latte => write!(f, "Latte"),
            ThemeVariant::Frappe => write!(f, "Frappe"),
            ThemeVariant::Macchiato => write!(f, "Macchiato"),
            ThemeVariant::Mocha => write!(f, "Mocha"),
        }
    }
}

impl ThemeVariant {
    pub fn theme(&self) -> &'static CatppuccinTheme {
        CatppuccinTheme::get_theme(self)
    }
}

#[derive(Clone, Debug)]
pub struct CatppuccinTheme {
    pub rosewater: &'static str,
    pub flamingo: &'static str,
    pub pink: &'static str,
    pub mauve: &'static str,
    pub red: &'static str,
    pub maroon: &'static str,
    pub peach: &'static str,
    pub yellow: &'static str,
    pub green: &'static str,
    pub teal: &'static str,
    pub sky: &'static str,
    pub sapphire: &'static str,
    pub blue: &'static str,
    pub lavender: &'static str,
    pub text: &'static str,
    pub subtext1: &'static str,
    pub subtext0: &'static str,
    pub overlay2: &'static str,
    pub overlay1: &'static str,
    pub overlay0: &'static str,
    pub surface2: &'static str,
    pub surface1: &'static str,
    pub surface0: &'static str,
    pub base: &'static str,
    pub mantle: &'static str,
    pub crust: &'static str,
}

pub const CATPPUCCIN_LATTE: CatppuccinTheme = CatppuccinTheme {
    rosewater: "#dc8a78",
    flamingo: "#dd7878",
    pink: "#ea76cb",
    mauve: "#8839ef",
    red: "#d20f39",
    maroon: "#e64553",
    peach: "#fe640b",
    yellow: "#df8e1d",
    green: "#40a02b",
    teal: "#179299",
    sky: "#04a5e5",
    sapphire: "#209fb5",
    blue: "#1e66f5",
    lavender: "#7287fd",
    text: "#4c4f69",
    subtext1: "#5c5f77",
    subtext0: "#6c6f85",
    overlay2: "#7c7f93",
    overlay1: "#8c8fa1",
    overlay0: "#9ca0b0",
    surface2: "#acb0be",
    surface1: "#bcc0cc",
    surface0: "#ccd0da",
    base: "#eff1f5",
    mantle: "#e6e9ef",
    crust: "#dce0e8",
};

pub const CATPPUCCIN_FRAPPE: CatppuccinTheme = CatppuccinTheme {
    rosewater: "#f2d5cf",
    flamingo: "#eebebe",
    pink: "#f4b8e4",
    mauve: "#ca9ee6",
    red: "#e78284",
    maroon: "#ea999c",
    peach: "#ef9f76",
    yellow: "#e5c890",
    green: "#a6d189",
    teal: "#81c8be",
    sky: "#99d1db",
    sapphire: "#85c1dc",
    blue: "#8caaee",
    lavender: "#babbf1",
    text: "#c6d0f5",
    subtext1: "#b5bfe2",
    subtext0: "#a5adce",
    overlay2: "#949cbb",
    overlay1: "#838ba7",
    overlay0: "#737994",
    surface2: "#626880",
    surface1: "#51576d",
    surface0: "#414559",
    base: "#303446",
    mantle: "#292c3c",
    crust: "#232634",
};

pub const CATPPUCCIN_MACCHIATO: CatppuccinTheme = CatppuccinTheme {
    rosewater: "#f4dbd6",
    flamingo: "#f0c6c6",
    pink: "#f5bde6",
    mauve: "#c6a0f6",
    red: "#ed8796",
    maroon: "#ee99a0",
    peach: "#f5a97f",
    yellow: "#eed49f",
    green: "#a6da95",
    teal: "#8bd5ca",
    sky: "#91d7e3",
    sapphire: "#7dc4e4",
    blue: "#8aadf4",
    lavender: "#b7bdf8",
    text: "#cad3f5",
    subtext1: "#b8c0e0",
    subtext0: "#a5adcb",
    overlay2: "#939ab7",
    overlay1: "#8087a2",
    overlay0: "#6e738d",
    surface2: "#5b6078",
    surface1: "#494d64",
    surface0: "#363a4f",
    base: "#24273a",
    mantle: "#1e2030",
    crust: "#181926",
};

pub const CATPPUCCIN_MOCHA: CatppuccinTheme = CatppuccinTheme {
    rosewater: "#f5e0dc",
    flamingo: "#f2cdcd",
    pink: "#f5c2e7",
    mauve: "#cba6f7",
    red: "#f38ba8",
    maroon: "#eba0ac",
    peach: "#fab387",
    yellow: "#f9e2af",
    green: "#a6e3a1",
    teal: "#94e2d5",
    sky: "#89dceb",
    sapphire: "#74c7ec",
    blue: "#89b4fa",
    lavender: "#b4befe",
    text: "#cdd6f4",
    subtext1: "#bac2de",
    subtext0: "#a6adc8",
    overlay2: "#9399b2",
    overlay1: "#7f849c",
    overlay0: "#6c7086",
    surface2: "#585b70",
    surface1: "#45475a",
    surface0: "#313244",
    base: "#1e1e2e",
    mantle: "#181825",
    crust: "#11111b",
};

impl CatppuccinTheme {
    pub fn get_theme(variant: &ThemeVariant) -> &'static CatppuccinTheme {
        match variant {
            ThemeVariant::Latte => &CATPPUCCIN_LATTE,
            ThemeVariant::Frappe => &CATPPUCCIN_FRAPPE,
            ThemeVariant::Macchiato => &CATPPUCCIN_MACCHIATO,
            ThemeVariant::Mocha => &CATPPUCCIN_MOCHA,
        }
    }

    pub fn apply_to_document(&self) {
        let document = web_sys::window().unwrap().document().unwrap();
        let html = document.document_element().unwrap();
        let html_element = html.dyn_ref::<web_sys::HtmlElement>().unwrap();
        
        html_element.style().set_property("--color-rosewater", self.rosewater).unwrap();
        html_element.style().set_property("--color-flamingo", self.flamingo).unwrap();
        html_element.style().set_property("--color-pink", self.pink).unwrap();
        html_element.style().set_property("--color-mauve", self.mauve).unwrap();
        html_element.style().set_property("--color-red", self.red).unwrap();
        html_element.style().set_property("--color-maroon", self.maroon).unwrap();
        html_element.style().set_property("--color-peach", self.peach).unwrap();
        html_element.style().set_property("--color-yellow", self.yellow).unwrap();
        html_element.style().set_property("--color-green", self.green).unwrap();
        html_element.style().set_property("--color-teal", self.teal).unwrap();
        html_element.style().set_property("--color-sky", self.sky).unwrap();
        html_element.style().set_property("--color-sapphire", self.sapphire).unwrap();
        html_element.style().set_property("--color-blue", self.blue).unwrap();
        html_element.style().set_property("--color-lavender", self.lavender).unwrap();
        html_element.style().set_property("--color-text", self.text).unwrap();
        html_element.style().set_property("--color-subtext1", self.subtext1).unwrap();
        html_element.style().set_property("--color-subtext0", self.subtext0).unwrap();
        html_element.style().set_property("--color-overlay2", self.overlay2).unwrap();
        html_element.style().set_property("--color-overlay1", self.overlay1).unwrap();
        html_element.style().set_property("--color-overlay0", self.overlay0).unwrap();
        html_element.style().set_property("--color-surface2", self.surface2).unwrap();
        html_element.style().set_property("--color-surface1", self.surface1).unwrap();
        html_element.style().set_property("--color-surface0", self.surface0).unwrap();
        html_element.style().set_property("--color-base", self.base).unwrap();
        html_element.style().set_property("--color-mantle", self.mantle).unwrap();
        html_element.style().set_property("--color-crust", self.crust).unwrap();
    }
}