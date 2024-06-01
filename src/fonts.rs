use eframe::egui::{self, FontDefinitions};

pub fn initialize_fonts() -> FontDefinitions {
    let font1 = egui::FontData::from_static(include_bytes!("assets/aclonica.ttf"));
    let font2 = egui::FontData::from_static(include_bytes!("assets/bellota.ttf"));

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("arial".into(), font1);
    fonts.font_data.insert("sans".into(), font2);

    fonts
        .families
        .insert(egui::FontFamily::Name("arial".into()), vec!["arial".into()]);

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "sans".into());

    fonts
}
