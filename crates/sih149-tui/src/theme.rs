use ratatui::style::Color;

pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // Backgrounds
    pub const BG: Color        = Color::Rgb(8, 10, 16);
    pub const SURFACE: Color   = Color::Rgb(16, 18, 28);
    pub const SURFACE2: Color  = Color::Rgb(22, 24, 36);
    pub const BORDER: Color    = Color::Rgb(38, 42, 62);
    pub const BORDER_HI: Color = Color::Rgb(60, 66, 96);

    // Accent palette
    pub const CYAN: Color      = Color::Rgb(56, 189, 248);   // primary
    pub const PURPLE: Color    = Color::Rgb(168, 85, 247);   // secondary
    pub const BLUE: Color      = Color::Rgb(96, 165, 250);
    pub const INDIGO: Color    = Color::Rgb(129, 140, 248);

    // Status
    pub const SUCCESS: Color   = Color::Rgb(52, 211, 153);
    pub const DANGER: Color    = Color::Rgb(248, 113, 113);
    pub const WARNING: Color   = Color::Rgb(251, 191, 36);
    pub const INFO: Color      = Color::Rgb(147, 197, 253);

    // Text
    pub const TEXT: Color      = Color::Rgb(226, 232, 240);
    pub const TEXT_DIM: Color  = Color::Rgb(148, 163, 184);
    pub const MUTED: Color     = Color::Rgb(71, 85, 105);
    pub const HIGHLIGHT: Color = Color::Rgb(255, 255, 255);

    // Entropy heatmap colors (low→high entropy)
    pub const ENT_0: Color = Color::Rgb(15, 23, 42);    // dead / zeroed
    pub const ENT_1: Color = Color::Rgb(20, 83, 45);    // near-zero (dark green)
    pub const ENT_2: Color = Color::Rgb(34, 197, 94);   // low (green)
    pub const ENT_3: Color = Color::Rgb(234, 179, 8);   // moderate (yellow)
    pub const ENT_4: Color = Color::Rgb(249, 115, 22);  // high (orange)
    pub const ENT_5: Color = Color::Rgb(239, 68, 68);   // encrypted/compressed (red)

    pub fn entropy_color(e: f64) -> Color {
        match e as u32 {
            0     => Self::ENT_0,
            1     => Self::ENT_1,
            2..=3 => Self::ENT_2,
            4..=5 => Self::ENT_3,
            6     => Self::ENT_4,
            _     => Self::ENT_5,
        }
    }

    pub fn entropy_bar_char(e: f64) -> char {
        // 8 levels of block characters
        let level = ((e / 8.0) * 8.0).round() as usize;
        ['░', '░', '▒', '▒', '▓', '▓', '█', '█', '█'][level.min(8)]
    }
}
