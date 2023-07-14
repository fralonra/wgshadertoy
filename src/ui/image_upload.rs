use egui::{vec2, Color32, CursorIcon, Image, Response, Sense, Stroke, TextureHandle, Ui, Vec2};

pub fn image_upload(ui: &mut Ui, size: f32, texture: Option<&TextureHandle>) -> Response {
    let response = ui.allocate_response(Vec2::splat(size), Sense::click());

    let rect = response.rect;

    let stroke = Stroke::new(1.0, Color32::from_gray(128));

    let painter = ui.painter_at(rect);

    match texture {
        Some(texture) => {
            let image = Image::new(texture, rect.size());
            image.paint_at(ui, rect);
        }
        None => {
            let center = rect.center();
            let half_len = size * 0.3;

            painter.line_segment(
                [center - vec2(0.0, half_len), center + vec2(0.0, half_len)],
                stroke,
            );
            painter.line_segment(
                [center - vec2(half_len, 0.0), center + vec2(half_len, 0.0)],
                stroke,
            );

            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }
        }
    }

    painter.rect_stroke(rect, 5.0, stroke);

    response
}
