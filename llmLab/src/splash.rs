use crate::branding::{
    gradient_color, is_center_glyph_zone, LOGO_ASCII, LOGO_TAGLINE, LOGO_UNICODE,
};
use crate::hardware::{format_gb, HardwareProfile};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub const SPLASH_TOTAL_MS: u64 = 1450;
const PHASE1_SCAN_MS: f32 = 620.0;
const PHASE1_PULSE_END_MS: f32 = 860.0;
const PHASE2_HW_END_MS: f32 = 1420.0;

pub fn render_splash(
    frame: &mut Frame<'_>,
    elapsed_ms: u64,
    use_utf8: bool,
    hw: &HardwareProfile,
) {
    let area = frame.area();
    let t = elapsed_ms as f32;

    let logo_lines = if use_utf8 { &LOGO_UNICODE } else { &LOGO_ASCII };
    let logo_height = logo_lines.len() as u16;
    let logo_width = 38u16;

    // Responsive placement: side-by-side on wide terminals, stacked on narrow consoles
    let side_by_side = area.width >= 94 && area.height >= 22;

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(if side_by_side { logo_height + 3 } else { logo_height + 12 }),
            Constraint::Min(1),
        ])
        .split(area);

    let center_block = outer[1];

    if side_by_side {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(2),
                Constraint::Length(logo_width),
                Constraint::Length(4),
                Constraint::Length(48),
                Constraint::Min(2),
            ])
            .split(center_block);

        render_logo_scan(frame, cols[1], t, logo_lines);
        if t >= PHASE1_SCAN_MS * 0.85 {
            render_hardware_reveal(frame, cols[3], t, hw);
        }
        render_tagline(frame, center_block, t);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(logo_height),
                Constraint::Length(2),
                Constraint::Length(9),
            ])
            .split(center_block);

        render_logo_scan(frame, rows[0], t, logo_lines);
        render_tagline(frame, rows[1], t);
        if t >= PHASE1_SCAN_MS * 0.85 {
            render_hardware_reveal(frame, rows[2], t, hw);
        }
    }

    // Subtle skip hint in bottom line
    if area.height > 4 {
        let hint_area = Rect::new(0, area.height - 1, area.width, 1);
        let hint = Paragraph::new("Enter / Space — пропустить")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, hint_area);
    }
}

fn render_logo_scan(frame: &mut Frame<'_>, area: Rect, elapsed_ms: f32, logo: &[&str; 18]) {
    let scan_progress = (elapsed_ms / PHASE1_SCAN_MS).clamp(0.0, 1.0);
    let visible_rows = (scan_progress * logo.len() as f32).ceil() as usize;

    // White pulse over the central </> cutout between 620ms and 860ms
    let pulse_active = elapsed_ms >= PHASE1_SCAN_MS && elapsed_ms <= PHASE1_PULSE_END_MS;
    let pulse_intensity = if pulse_active {
        let mid = (PHASE1_SCAN_MS + PHASE1_PULSE_END_MS) * 0.5;
        let half = (PHASE1_PULSE_END_MS - PHASE1_SCAN_MS) * 0.5;
        1.0 - ((elapsed_ms - mid).abs() / half).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut rendered_lines = Vec::with_capacity(logo.len());
    for (row_idx, &line) in logo.iter().enumerate() {
        if row_idx >= visible_rows {
            rendered_lines.push(Line::from(""));
            continue;
        }
        let row_t = row_idx as f32 / (logo.len().saturating_sub(1).max(1) as f32);
        let base_color = gradient_color(row_t);

        let is_scan_head = scan_progress < 1.0 && row_idx + 1 == visible_rows;
        let mut spans = Vec::new();

        for (col_idx, ch) in line.chars().enumerate() {
            let color = if is_scan_head {
                Color::White
            } else if pulse_intensity > 0.15 && is_center_glyph_zone(row_idx, col_idx) {
                blend_to_white(base_color, pulse_intensity)
            } else {
                base_color
            };
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        rendered_lines.push(Line::from(spans));
    }

    let p = Paragraph::new(rendered_lines).alignment(Alignment::Center);
    frame.render_widget(p, area);
}

fn render_tagline(frame: &mut Frame<'_>, area: Rect, elapsed_ms: f32) {
    if elapsed_ms < PHASE1_SCAN_MS * 0.7 || area.height < 2 {
        return;
    }
    let y = area.y + area.height.saturating_sub(2);
    let tag_area = Rect::new(area.x, y, area.width, 1);
    let p = Paragraph::new(LOGO_TAGLINE)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Rgb(165, 220, 255))
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(p, tag_area);
}

fn render_hardware_reveal(
    frame: &mut Frame<'_>,
    area: Rect,
    elapsed_ms: f32,
    hw: &HardwareProfile,
) {
    let hw_progress =
        ((elapsed_ms - PHASE1_SCAN_MS * 0.85) / (PHASE2_HW_END_MS - PHASE1_SCAN_MS * 0.85))
            .clamp(0.0, 1.0);

    let simd = if hw.cpu.avx2 {
        "AVX2"
    } else if hw.cpu.neon {
        "NEON"
    } else {
        "Scalar"
    };

    let cores_text = if hw.cpu.e_cores > 0 {
        format!(
            "{} P-ядер / {} E-ядер ({} потоков) · {}",
            hw.cpu.p_cores, hw.cpu.e_cores, hw.cpu.logical_threads, simd
        )
    } else {
        format!(
            "{} физ. ядер ({} потоков) · {}",
            hw.cpu.p_cores, hw.cpu.logical_threads, simd
        )
    };

    let accel_text = format!(
        "Vulkan: {} · OpenVINO: {}",
        if hw.gpu.vulkan_available { "Да" } else { "Нет" },
        if hw.gpu.openvino_available { "Да" } else { "Нет" },
    );

    let rows = [
        ("CPU", hw.cpu.brand.clone(), Color::Rgb(0, 245, 212)),
        ("Ядра", cores_text, Color::Rgb(150, 220, 255)),
        (
            "GPU",
            format!("{} ({} · {} МБ VRAM)", hw.gpu.name, hw.gpu.vendor, hw.gpu.vram_mb),
            Color::Rgb(58, 134, 255),
        ),
        ("Ускор.", accel_text, Color::Rgb(170, 200, 255)),
        (
            "RAM",
            format!(
                "{} / {} (до ~{} под GGUF)",
                format_gb(hw.ram_used_bytes),
                format_gb(hw.ram_total_bytes),
                format_gb(hw.recommended_model_budget_bytes),
            ),
            Color::Rgb(120, 255, 190),
        ),
    ];

    let visible_count = (hw_progress * rows.len() as f32).ceil() as usize;
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Системный профиль",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (idx, (label, val, color)) in rows.iter().enumerate() {
        if idx >= visible_count {
            break;
        }
        lines.push(Line::from(vec![
            Span::styled(
                format!("{label:<7} "),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(val.clone(), Style::default().fg(*color)),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(36, 70, 110)))
        .title(" Hardware Probe ");
    let widget = Paragraph::new(lines).block(block);
    frame.render_widget(widget, area);
}

fn blend_to_white(base: Color, factor: f32) -> Color {
    match base {
        Color::Rgb(r, g, b) => {
            let f = factor.clamp(0.0, 1.0);
            let nr = (r as f32 + (255.0 - r as f32) * f).round() as u8;
            let ng = (g as f32 + (255.0 - g as f32) * f).round() as u8;
            let nb = (b as f32 + (255.0 - b as f32) * f).round() as u8;
            Color::Rgb(nr, ng, nb)
        }
        other => other,
    }
}
