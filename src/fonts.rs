use std::sync::Arc;

use eframe::egui::{self, Context, FontDefinitions, Style, TextStyle};
use eyre::Result;

pub fn initialize_fonts() -> Result<FontDefinitions> {
    let arial = egui::FontData::from_static(include_bytes!("assets/aclonica.ttf"));
    let sans = egui::FontData::from_static(include_bytes!("assets/bellota.ttf"));
    let mono = egui::FontData::from_static(include_bytes!("assets/noto-sans-mono.ttf"));

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("arial".into(), arial);
    fonts.font_data.insert("sans".into(), sans);
    fonts.font_data.insert("monospace".into(), mono);

    fonts
        .families
        .insert(egui::FontFamily::Name("arial".into()), vec!["arial".into()]);

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .ok_or(eyre::eyre!["couldn't load proportional font"])?
        .insert(0, "sans".into());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .ok_or(eyre::eyre!["couldn't load monospace font"])?
        .insert(0, "monospace".into());

    Ok(fonts)
}

pub fn set_font_size(ctx: &Context, size: f32) {
    let mut style = ctx.style().as_ref().clone();

    // Update font size for specific text styles
    for text_style in &[
        TextStyle::Body,
        TextStyle::Heading,
        TextStyle::Button,
        TextStyle::Monospace,
    ] {
        if let Some(font_id) = style.text_styles.get_mut(text_style) {
            font_id.size = size;
        }
    }

    // Apply the modified style
    ctx.set_style(<Style as Into<Arc<Style>>>::into(style.clone()));
}
