use palette::{FromColor, Hsl, Srgb};
use rand::{rngs::SmallRng, RngCore, SeedableRng};

fn print_next(rng: &mut SmallRng) {
    let hue = rng.next_u32() % 360;

    println!("Rand: {}", hue);

    let random_hsl = Hsl::new(hue as f32, 0.5, 0.5);
    let random_rgb = Srgb::from_color(random_hsl);

    println!(
        "{} {} {}",
        random_rgb.red, random_rgb.green, random_rgb.blue
    )
}

#[test]
fn test_color_randomise() {
    let mut rng = SmallRng::from_entropy();

    print_next(&mut rng);
    print_next(&mut rng);
    print_next(&mut rng);
    print_next(&mut rng);
    print_next(&mut rng);
}
