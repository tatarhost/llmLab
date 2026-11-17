use ratatui::style::Color;

pub const LOGO_UNICODE: [&str; 18] = [
    "█████████████▀        ▀█████████████",
    "████████████▀          ▀████████████",
    "████▀                           ████",
    "█████                         ▄█████",
    " ▀████▄▄▄██████████████████▄▄█████▀ ",
    "   ▀███████████████   ▄█████████▀   ",
    "     ▀████▀ ▀█████    █▀ ▀████▀     ",
    "     ▄██▀    ▄███▀   █▄    ▀██▄     ",
    "   ▄███    ▄████▀   ████▄    ███▄   ",
    " ▄█████▄    ▀███   ▄███▀    ▄█████▄ ",
    "█████████▄    █   ▄███    ▄█████████",
    "███████████▄▄█    █████▄▄███████████",
    "██████████████▄▄▄███████████████████",
    "██████████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀██████████",
    "█████████                  █████████",
    "████████                    ████████",
    "███████                      ███████",
    "██████                        ██████",
];

pub const LOGO_ASCII: [&str; 18] = [
    "#############         #############",
    "############           ############",
    "####                           ####",
    "#####                         #####",
    "  ####___##################__##### ",
    "   ###############   ##########    ",
    "     #####  #####     #  #####     ",
    "     ###     ####    #     ###     ",
    "   ####    #####    ####    ####   ",
    " ######     ###    ####     ###### ",
    "#########    #    ####    #########",
    "###########__#    #####__##########",
    "##############___##################",
    "##########----------------#########",
    "#########                 #########",
    "########                   ########",
    "#######                     #######",
    "######                       ######",
];

pub const LOGO_TAGLINE: &str = "l l m L a b   //   L O C A L   A I   R U N T I M E";

/// Linearly interpolates between Cyan (#00F5D4) and Neon Blue (#3A86FF).
pub fn gradient_color(progress: f32) -> Color {
    let t = progress.clamp(0.0, 1.0);
    let r = (0.0 + (58.0 - 0.0) * t).round() as u8;
    let g = (245.0 + (134.0 - 245.0) * t).round() as u8;
    let b = (212.0 + (255.0 - 212.0) * t).round() as u8;
    Color::Rgb(r, g, b)
}

/// Returns true if the character coordinate belongs to the central `</>` visor cutout zone.
pub fn is_center_glyph_zone(row: usize, col: usize) -> bool {
    (5..=12).contains(&row) && (11..=25).contains(&col)
}

/// Detects whether the current console supports UTF-8 half-block rendering.
/// On Windows, attempts to switch console output code page to 65001 (UTF-8) if running
/// in a modern host, or falls back to ASCII on legacy cmd.exe without UTF-8 support.
pub fn detect_utf8_support(force_ascii: bool) -> bool {
    if force_ascii {
        return false;
    }

    #[cfg(windows)]
    {
        use windows::Win32::System::Console::{GetConsoleOutputCP, SetConsoleOutputCP};

        let modern_term = std::env::var_os("WT_SESSION").is_some()
            || std::env::var_os("TERM_PROGRAM").is_some()
            || std::env::var_os("ConEmuANSI").is_some();

        unsafe {
            let cp = GetConsoleOutputCP();
            if cp == 65001 {
                return true;
            }
            if modern_term && SetConsoleOutputCP(65001).is_ok() {
                return true;
            }
            // Attempt UTF-8 codepage activation for standard Windows 10/11 consoles
            if SetConsoleOutputCP(65001).is_ok() {
                return GetConsoleOutputCP() == 65001;
            }
            false
        }
    }

    #[cfg(not(windows))]
    {
        let lang = std::env::var("LANG").unwrap_or_default();
        let lc_all = std::env::var("LC_ALL").unwrap_or_default();
        lang.to_uppercase().contains("UTF-8")
            || lc_all.to_uppercase().contains("UTF-8")
            || std::env::var_os("TERM").is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_dimensions_are_consistent() {
        assert_eq!(LOGO_UNICODE.len(), 18);
        assert_eq!(LOGO_ASCII.len(), 18);
        for line in LOGO_UNICODE {
            assert_eq!(line.chars().count(), 36);
        }
    }

    #[test]
    fn gradient_endpoints_match_brand_palette() {
        assert_eq!(gradient_color(0.0), Color::Rgb(0, 245, 212));
        assert_eq!(gradient_color(1.0), Color::Rgb(58, 134, 255));
    }

    #[test]
    fn force_ascii_overrides_detection() {
        assert!(!detect_utf8_support(true));
    }
}
