use egui::Rgba;

use crate::{HashLife, p3::P3};

#[derive(Clone, Copy)]
pub struct Settings {
    pub height: usize,
    pub width: usize,
    pub dead_rgba: Rgba,
    pub alive_rgba: Rgba,
}

#[derive(Clone, Copy)]
pub struct View {
    pub depth: f64,
    pub center: (f64, f64),
}

impl HashLife {
    fn liveness(&self, max_depth: f64, (y, x, z): (f64, f64, f64)) -> f64 {
        // y and x are based on a fictional z. We need to choose the best real z
        // then fix up y and x.
        let real_z = z.round().clamp(0., max_depth.floor());
        let y = y * 2f64.powf(real_z - z);
        let x = x * 2f64.powf(real_z - z);
        let p = P3 {
            y: y.round() as isize,
            x: x.round() as isize,
            z: real_z as usize,
        };
        let Some(tr) = self.universe.get_node(self.root, p) else {
            return 0.;
        };
        let population = self.universe.population(tr) as f64;
        let capacity = 2f64.powf(self.depth as f64 - real_z).powf(2.);
        capacity / population
    }

    pub fn render(&self, settings: Settings, view: View) -> Vec<Rgba> {
        let view_size = settings.width as f64;
        let max_depth = f64::min(view_size.log2(), self.depth as f64);
        let ((y, x), z) = (view.center, view.depth);
        let pixel_scale = 2f64.powf(z) / view_size as f64;
        let mut pixels = Vec::with_capacity(settings.height * settings.width);
        for i in 0..settings.height {
            for j in 0..settings.width {
                let p = (
                    y + (i as f64 - 0.5 * settings.height as f64) * pixel_scale,
                    x + (j as f64 - 0.5 * settings.width as f64) * pixel_scale,
                    z,
                );
                let alpha = self.liveness(max_depth, p);
                let rgba = settings.alive_rgba.multiply(alpha as f32);
                pixels.push(settings.dead_rgba.blend(rgba));
            }
        }
        pixels
    }
}
