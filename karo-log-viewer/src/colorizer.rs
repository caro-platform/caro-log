use std::collections::HashMap;

use palette::{FromColor, Hsl, Srgb};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use termion::color;

const LIGHT_GREY: color::Rgb = color::Rgb(120, 120, 120);

pub struct Colorizer {
    service_colors: HashMap<String, color::Rgb>,
    rng: SmallRng,
}

impl Colorizer {
    pub fn new() -> Self {
        Self {
            service_colors: HashMap::new(),
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn colorize(&mut self, log_line: &String) -> String {
        if log_line == "\n" {
            return log_line.clone();
        }

        let components: Vec<&str> = log_line.splitn(7, ' ').collect();

        if components.len() != 7 {
            eprintln!("Failed to split log line");
            return log_line.clone();
        }

        format!(
            "{} {} {} {} {} {} {}",
            components[0],
            components[1],
            self.colorize_service(components[2]),
            self.colorize_level(components[3]),
            self.dim(components[4]),
            self.dim(components[5]),
            components[6]
        )
    }

    fn colorize_service(&mut self, service_name: &str) -> String {
        let fg_color = if let Some(color) = self.service_colors.get(service_name) {
            color.clone()
        } else {
            let random_color = self.randomize_service_color();

            self.service_colors
                .insert(service_name.into(), random_color);

            random_color
        };

        let name_components: Vec<&str> = service_name.splitn(2, '#').collect();

        // Succesfully split service_name and pid
        if name_components.len() == 2 {
            let _darker_rgb = color::Rgb(
                fg_color.0.saturating_sub(40),
                fg_color.1.saturating_sub(40),
                fg_color.2.saturating_sub(60),
            );

            format!(
                "{}{}{}#{}",
                fg_color.fg_string(),
                name_components[0],
                color::Fg(color::Reset),
                name_components[1],
            )
        } else {
            format!(
                "{}{}{}",
                fg_color.fg_string(),
                service_name,
                color::Fg(color::Reset)
            )
        }
    }

    fn randomize_service_color(&mut self) -> color::Rgb {
        let hue = self.rng.next_u32() % 360;
        let random_hsl = Hsl::new(hue as f32, 0.8, 0.4);
        let random_rgb = Srgb::from_color(random_hsl);

        color::Rgb(
            (random_rgb.red * 256.) as u8,
            (random_rgb.green * 256.) as u8,
            (random_rgb.blue * 256.) as u8,
        )
    }

    fn colorize_level(&self, level: &str) -> String {
        match level {
            "[ERROR]" => format!(
                "{}[ERROR]{}",
                color::Fg(color::Red),
                color::Fg(color::Reset)
            ),
            "[WARNING]" => format!(
                "{}[WARNING]{}",
                color::Fg(color::Yellow),
                color::Fg(color::Reset)
            ),
            "[INFO]" => format!(
                "{}[INFO]{}",
                color::Fg(color::Green),
                color::Fg(color::Reset)
            ),
            "[DEBUG]" => format!(
                "{}[DEBUG]{}",
                color::Fg(color::Blue),
                color::Fg(color::Reset)
            ),
            s => format!(
                "{}{}{}",
                color::Fg(color::White),
                s,
                color::Fg(color::Reset)
            ),
        }
    }

    fn dim(&self, string: &str) -> String {
        format!(
            "{}{}{}",
            LIGHT_GREY.fg_string(),
            string,
            color::Fg(color::Reset)
        )
    }
}
