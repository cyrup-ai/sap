use crossterm::style::Color;

/// Convert a crossterm Color to RGB components
/// Returns (r, g, b) as u8 values 0-255
fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (128, 128, 128),
        Color::Red => (255, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::Green => (0, 255, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::Yellow => (255, 255, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::Blue => (0, 0, 255),
        Color::DarkBlue => (0, 0, 128),
        Color::Magenta => (255, 0, 255),
        Color::DarkMagenta => (128, 0, 128),
        Color::Cyan => (0, 255, 255),
        Color::DarkCyan => (0, 128, 128),
        Color::White => (255, 255, 255),
        Color::Grey => (192, 192, 192),
        Color::Reset => (255, 255, 255), // Default to white for reset
        Color::AnsiValue(_) => (128, 128, 128), // Default to grey for ANSI colors
    }
}

/// Apply alpha blending between two RGB colors
/// Formula: result = foreground * alpha + background * (1 - alpha)
fn alpha_blend_rgb(fg_rgb: (u8, u8, u8), bg_rgb: (u8, u8, u8), alpha: f32) -> (u8, u8, u8) {
    let alpha = alpha.clamp(0.0, 1.0);
    let inv_alpha = 1.0 - alpha;
    
    let r = (fg_rgb.0 as f32 * alpha + bg_rgb.0 as f32 * inv_alpha).round() as u8;
    let g = (fg_rgb.1 as f32 * alpha + bg_rgb.1 as f32 * inv_alpha).round() as u8;
    let b = (fg_rgb.2 as f32 * alpha + bg_rgb.2 as f32 * inv_alpha).round() as u8;
    
    (r, g, b)
}

/// Create a muted version of a foreground color by blending with a background color
pub fn mute_color(foreground: Color, background: Color, alpha: f32) -> Color {
    let fg_rgb = color_to_rgb(foreground);
    let bg_rgb = color_to_rgb(background);
    let (r, g, b) = alpha_blend_rgb(fg_rgb, bg_rgb, alpha);
    
    Color::Rgb { r, g, b }
}