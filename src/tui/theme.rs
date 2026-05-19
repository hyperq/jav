#![allow(dead_code)]
use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode { Dark, Light, Auto }

#[derive(Clone)]
pub struct Theme {
    pub rosewater: Color,
    pub flamingo: Color,
    pub pink: Color,
    pub mauve: Color,
    pub red: Color,
    pub maroon: Color,
    pub peach: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub sky: Color,
    pub sapphire: Color,
    pub blue: Color,
    pub lavender: Color,
    pub text: Color,
    pub subtext1: Color,
    pub subtext0: Color,
    pub overlay2: Color,
    pub overlay1: Color,
    pub overlay0: Color,
    pub surface2: Color,
    pub surface1: Color,
    pub surface0: Color,
    pub base: Color,
    pub mantle: Color,
    pub crust: Color,
}

impl Theme {
    pub fn frappe() -> Self {
        Self {
            rosewater: Color::Rgb(242, 213, 207),
            flamingo:  Color::Rgb(238, 190, 190),
            pink:      Color::Rgb(244, 184, 228),
            mauve:     Color::Rgb(202, 158, 230),
            red:       Color::Rgb(231, 130, 132),
            maroon:    Color::Rgb(234, 153, 156),
            peach:     Color::Rgb(239, 159, 118),
            yellow:    Color::Rgb(229, 200, 144),
            green:     Color::Rgb(166, 209, 137),
            teal:      Color::Rgb(129, 200, 190),
            sky:       Color::Rgb(153, 209, 219),
            sapphire:  Color::Rgb(133, 193, 220),
            blue:      Color::Rgb(140, 170, 238),
            lavender:  Color::Rgb(186, 187, 241),
            text:      Color::Rgb(198, 208, 245),
            subtext1:  Color::Rgb(181, 191, 226),
            subtext0:  Color::Rgb(165, 173, 206),
            overlay2:  Color::Rgb(148, 156, 187),
            overlay1:  Color::Rgb(131, 139, 167),
            overlay0:  Color::Rgb(115, 121, 148),
            surface2:  Color::Rgb(98, 104, 128),
            surface1:  Color::Rgb(81, 87, 109),
            surface0:  Color::Rgb(65, 69, 89),
            base:      Color::Rgb(48, 52, 70),
            mantle:    Color::Rgb(41, 44, 60),
            crust:     Color::Rgb(35, 38, 52),
        }
    }

    pub fn latte() -> Self {
        Self {
            rosewater: Color::Rgb(220, 138, 120),
            flamingo:  Color::Rgb(221, 120, 120),
            pink:      Color::Rgb(234, 118, 203),
            mauve:     Color::Rgb(136, 57, 239),
            red:       Color::Rgb(210, 15, 57),
            maroon:    Color::Rgb(230, 69, 83),
            peach:     Color::Rgb(254, 100, 11),
            yellow:    Color::Rgb(223, 142, 29),
            green:     Color::Rgb(64, 160, 43),
            teal:      Color::Rgb(23, 146, 153),
            sky:       Color::Rgb(4, 165, 229),
            sapphire:  Color::Rgb(32, 159, 181),
            blue:      Color::Rgb(30, 102, 245),
            lavender:  Color::Rgb(114, 135, 253),
            text:      Color::Rgb(76, 79, 105),
            subtext1:  Color::Rgb(92, 95, 119),
            subtext0:  Color::Rgb(108, 111, 133),
            overlay2:  Color::Rgb(124, 127, 147),
            overlay1:  Color::Rgb(140, 143, 161),
            overlay0:  Color::Rgb(156, 160, 176),
            surface2:  Color::Rgb(172, 176, 190),
            surface1:  Color::Rgb(188, 192, 204),
            surface0:  Color::Rgb(204, 208, 218),
            base:      Color::Rgb(239, 241, 245),
            mantle:    Color::Rgb(230, 233, 239),
            crust:     Color::Rgb(220, 224, 232),
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        Self::from_mode_with_luma(mode, None)
    }

    pub fn from_mode_with_luma(mode: ThemeMode, luma: Option<f32>) -> Self {
        match mode {
            ThemeMode::Dark => Self::frappe(),
            ThemeMode::Light => Self::latte(),
            ThemeMode::Auto => Self::from_mode(ThemeMode::from_luma(luma)),
        }
    }
}

impl ThemeMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Auto => Self::Dark,
            Self::Dark => Self::Light,
            Self::Light => Self::Auto,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "跟随终端",
            Self::Dark => "Dark (Frappé)",
            Self::Light => "Light (Latte)",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            "auto" => Self::Auto,
            _ => Self::Auto,
        }
    }

    pub fn detect() -> Self {
        Self::from_luma(terminal_light::luma().ok())
    }

    pub fn from_luma(luma: Option<f32>) -> Self {
        match luma {
            Some(l) if l > 0.6 => Self::Light,
            _ => Self::Dark,
        }
    }
}
