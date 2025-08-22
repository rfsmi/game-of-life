use std::str::FromStr;

use eframe::{CreationContext, Frame};
use egui::{
    Color32, ColorImage, Context, Image, Rgba, TextureHandle, TextureOptions, Ui,
    load::SizedTexture,
};
use hashlife::{HashLife, render};

pub struct App {
    hl: HashLife,
    log_2_steps: usize,
    texture: Option<TextureHandle>,
    view: render::View,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        // let mut hl = HashLife::from_str(
        //     "
        //         oo       o
        //         o o       o
        //          o      ooo",
        // )
        // .unwrap();
        // hl.step(10);
        let mut hl = HashLife::new();
        for y in -100..100isize {
            for x in -100..100isize {
                if x.rem_euclid(2) == 0 && y.rem_euclid(2) == 0 {
                    hl.set_bit((y, x));
                }
            }
        }
        Self {
            hl,
            log_2_steps: 0,
            texture: None,
            view: render::View {
                center: (0., 0.),
                zoom: 0.5,
            },
        }
    }

    fn render_hashlife(&mut self, ctx: &Context, ui: &mut Ui) -> Image<'_> {
        let rect = ui.max_rect();
        let settings = render::Settings {
            height: rect.height() as usize,
            width: rect.width() as usize,
            cell_size: 1.,
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
        self.texture = Some(texture);
        Image::new(sized_texture).fit_to_exact_size(size)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.log_2_steps, 0..=1000).logarithmic(true));
                if ui.button("Step (log 2)").clicked() {
                    self.hl.step(self.log_2_steps);
                }
            });
            ui.add(
                egui::Slider::new(&mut self.view.zoom, 1e-10..=2.)
                    .logarithmic(true)
                    .show_value(false)
                    .text("Zoom"),
            );
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let image = self.render_hashlife(ctx, ui);
            ui.add(image);
        });
    }
}
