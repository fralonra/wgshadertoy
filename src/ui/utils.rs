use egui::{vec2, widget_text::WidgetTextGalley, NumExt, TextStyle, Ui, Vec2, WidgetText};

pub fn layout_text_widget(
    ui: &mut Ui,
    text: impl Into<WidgetText>,
    padding: Vec2,
) -> (WidgetTextGalley, Vec2) {
    let total_padding = padding + padding;

    let text: WidgetText = text.into();
    let text = text.into_galley(
        ui,
        None,
        ui.available_width() - total_padding.x,
        TextStyle::Button,
    );

    let widget_size = vec2(
        text.size().x + total_padding.x,
        (text.size().y + total_padding.y).at_least(ui.spacing().interact_size.y),
    );

    (text, widget_size)
}
