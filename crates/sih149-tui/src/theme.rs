use ratatui::style::Color;

pub struct Theme;
impl Theme {
    pub const BG: Color = Color::Rgb(10, 12, 18);
    pub const SURFACE: Color = Color::Rgb(18, 20, 30);
    pub const BORDER: Color = Color::Rgb(40, 44, 66);
    pub const ACCENT: Color = Color::Rgb(82, 196, 255); // cyan
    pub const ACCENT2: Color = Color::Rgb(168, 85, 247); // purple
    pub const SUCCESS: Color = Color::Rgb(52, 211, 153);
    pub const DANGER: Color = Color::Rgb(248, 113, 113);
    pub const WARNING: Color = Color::Rgb(251, 191, 36);
    pub const TEXT: Color = Color::Rgb(226, 232, 240);
    pub const MUTED: Color = Color::Rgb(100, 116, 139);
    pub const HIGHLIGHT: Color = Color::Rgb(255, 255, 255);
}
