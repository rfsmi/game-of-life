use std::str::FromStr;

use eframe::{CreationContext, Frame};
use egui::{
    Color32, ColorImage, Context, Image, Rgba, TextureHandle, TextureOptions, Ui,
    load::SizedTexture,
};
use hashlife::{HashLife, render};

pub struct App {
    hl: HashLife,
    texture: Option<TextureHandle>,
    view: render::View,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        Self {
            hl: HashLife::from_str(
                "
                oo       o
                o o       o
                 o      ooo",
            )
            .unwrap(),
            texture: None,
            view: render::View {
                center: (0., 0.),
                depth: 4.,
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let settings = render::Settings {
                height: ui.max_rect().width() as usize,
                width: ui.max_rect().height() as usize,
                dead_rgba: Rgba::WHITE,
                alive_rgba: Rgba::BLACK,
            };
            let pixels = self.hl.render(settings, self.view);
            let pixels: Vec<Color32> = pixels.into_iter().map(From::from).collect();
            let image = ColorImage::new([settings.width, settings.height], pixels);
            let options = TextureOptions::NEAREST;
            let texture = match self.texture.take() {
                Some(mut t) if t.size() == image.size => {
                    t.set(image, options);
                    t
                }
                _ => ctx.load_texture("hashlife", image, options),
            };
            let size = texture.size_vec2();
            let sized_texture = SizedTexture::new(&texture, size);
            ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
            self.texture = Some(texture);
        });
    }
}
