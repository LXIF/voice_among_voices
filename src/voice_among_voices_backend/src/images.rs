use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke};

use crate::structs::VoiceNodeEgress;

pub fn generate_nft_image(nodes: &[VoiceNodeEgress], nft_id: u32) -> Vec<u8> {
    let mut pixmap = Pixmap::new(1200, 1200).unwrap();

    // Set white background
    pixmap.fill(tiny_skia::Color::WHITE);

    let center_x = 600.0;
    let center_y = 600.0;
    let radius = 450.0;
    let mut stroke = Stroke::default();
    stroke.width = 2.;

    // Draw lines
    for angle in 0..360 {
        let rad = (90.0 - angle as f32).to_radians();
        let is_selected = nft_id == angle;

        let x1 = center_x - rad.cos() * radius * if is_selected { 0.3 } else { 1.0 };
        let y1 = center_y + rad.sin() * radius * if is_selected { 0.3 } else { 1.0 };
        let x2 = center_x - rad.cos() * radius * if is_selected { 1.2 } else { 1.1 };
        let y2 = center_y + rad.sin() * radius * if is_selected { 1.2 } else { 1.1 };

        let mut paint = Paint::default();
        let (r, g, b) = hsv_to_rgb(angle as f32, 100.0, 100.0);
        paint.set_color(tiny_skia::Color::from_rgba8(r, g, b, 255));
        paint.anti_alias = true;

        let mut path = PathBuilder::new();
        path.move_to(x1, y1);
        path.line_to(x2, y2);

        pixmap.stroke_path(
            &path.finish().unwrap(),
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            None,
        );
    }

    // Draw nodes
    for node in nodes {
        let is_selected = node.id == nft_id as usize;
        let x = center_x + node.x as f32 * 10.0;
        let y = center_y - node.y as f32 * 10.0;
        let radius = node.radius as f32 * 10.0;

        let mut paint = Paint::default();
        let (r, g, b) = hsv_to_rgb(node.id as f32, 100.0, 100.0);
        paint.set_color(tiny_skia::Color::from_rgba8(r, g, b, 255));
        paint.anti_alias = true;

        let path = PathBuilder::from_circle(x, y, radius).unwrap();

        pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            None,
        );

        if is_selected {
            let mut fill_paint = Paint::default();
            let (r, g, b) = hsv_to_rgb(node.id as f32, 100.0, 100.0);
            fill_paint.set_color(tiny_skia::Color::from_rgba8(r, g, b, 255));
            fill_paint.anti_alias = true;

            let mut fill_path = PathBuilder::new();
            fill_path.push_circle(x, y, radius);
            pixmap.fill_path(
                &fill_path.finish().unwrap(),
                &fill_paint,
                tiny_skia::FillRule::Winding,
                tiny_skia::Transform::identity(),
                None,
            );

            // Additional circles for selected node
            for offset in 1..=2 {
                let mut path = PathBuilder::new();
                path.push_circle(x, y, radius + offset as f32 * 10.0);

                let opacity = ((1.0 - offset as f32 * 0.33) * 255.0) as u8;
                let mut paint = Paint::default();
                let (r, g, b) = hsv_to_rgb(node.id as f32, 100.0, 100.0);
                paint.set_color(tiny_skia::Color::from_rgba8(r, g, b, opacity));
                paint.anti_alias = true;

                pixmap.stroke_path(
                    &path.finish().unwrap(),
                    &paint,
                    &stroke,
                    tiny_skia::Transform::identity(),
                    None,
                );
            }
        }
    }

    // Convert to PNG
    pixmap.encode_png().unwrap()
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let s = s / 100.0;
    let v = v / 100.0;
    let i = (h / 60.0).floor() as i32;
    let f = h / 60.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (0.0, 0.0, 0.0),
    };

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}
