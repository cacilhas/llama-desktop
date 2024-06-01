use eframe::egui::{self, FontDefinitions};

pub fn initialize_fonts() -> FontDefinitions {
    let arial = egui::FontData::from_static(include_bytes!("assets/aclonica.ttf"));
    let sans = egui::FontData::from_static(include_bytes!("assets/bellota.ttf"));
    let mono = egui::FontData::from_static(include_bytes!("assets/firacode.ttf"));

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("arial".into(), arial);
    fonts.font_data.insert("sans".into(), sans);
    fonts.font_data.insert("monospace".into(), mono);

    fonts
        .families
        .insert(egui::FontFamily::Name("arial".into()), vec!["arial".into()]);

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "sans".into());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "monospace".into());

    fonts
}
