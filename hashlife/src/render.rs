use egui::Rgba;

use crate::{HashLife, p3::P3};

#[derive(Clone, Copy)]
pub struct Settings {
    pub height: usize,
    pub width: usize,
    pub cell_size: f64,
    pub dead_rgba: Rgba,
    pub alive_rgba: Rgba,
}

#[derive(Clone, Copy)]
pub struct View {
    pub zoom: f64,
    pub center: (f64, f64),
}

impl HashLife {
    fn liveness(&self, (y, x, z): (f64, f64, f64)) -> f64 {
        let real_z = z.round().max(0.);
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
        if population == 0. {
            return 0.;
        }
        let capacity = 2f64.powf(self.depth as f64 - real_z).powf(2.);
        population / capacity
    }

    pub fn render(&self, settings: Settings, view: View) -> Vec<Rgba> {
        let mut pixels = Vec::with_capacity(settings.height * settings.width);
        let (y, x) = view.center;
        let mut z = self.depth as f64;
        let mut pixels_per_cell = view.zoom * settings.cell_size;
        if pixels_per_cell < 1.0 {
            z += pixels_per_cell.log2();
            pixels_per_cell = 1.0;
        }
        let (y, x) = (
            y - settings.height as f64 / pixels_per_cell / 2.,
            x - settings.width as f64 / pixels_per_cell / 2.,
        );
        for i in 0..settings.height {
            for j in 0..settings.width {
                let p = (
                    y + i as f64 / pixels_per_cell,
                    x + j as f64 / pixels_per_cell,
                    z,
                );
                let alpha = self.liveness(p);
                let rgba = settings.alive_rgba.multiply(alpha as f32);
                pixels.push(settings.dead_rgba.blend(rgba));
            }
        }
        pixels
    }
}
