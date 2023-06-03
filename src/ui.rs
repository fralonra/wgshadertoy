mod highlight;
mod image_upload;

use crate::event::{AppStatus, UserEvent};
use egui::{
    style::FontSelection, widgets, Align, CentralPanel, ClippedPrimitive, Color32, ColorImage,
    Context, ImageData, Layout, ScrollArea, TextEdit, TextureFilter, TextureHandle, TexturesDelta,
    TopBottomPanel,
};
use egui_winit::State;
use highlight::{CodeTheme, Highlighter};
use image_upload::image_upload;
use winit::{event::WindowEvent, event_loop::EventLoopProxy, window::Window};

pub struct EditContext {
    pub frag: String,
    pub name: String,
}

pub struct Ui {
    app_status: Option<(AppStatus, String)>,
    context: Context,
    highlighter: Highlighter,
    state: State,
    textures: Vec<TextureHandle>,
}

impl Ui {
    pub fn new(state: State) -> Self {
        let context = egui::Context::default();

        Self {
            app_status: None,
            context,
            highlighter: Highlighter::default(),
            state,
            textures: vec![],
        }
    }

    pub fn add_texture(&mut self, width: u32, height: u32, data: &[u8]) {
        self.textures.push(self.context.load_texture(
            "debug",
            ImageData::Color(ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                &data,
            )),
            TextureFilter::Linear,
        ));
    }

    pub fn change_status(&mut self, status: Option<(AppStatus, String)>) {
        self.app_status = status;
    }

    pub fn change_texture(&mut self, index: usize, width: u32, height: u32, data: &[u8]) {
        self.textures[index] = self.context.load_texture(
            "debug",
            ImageData::Color(ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                &data,
            )),
            TextureFilter::Linear,
        );
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.state.on_event(&self.context, event);
    }

    pub fn prepare(
        &mut self,
        window: &Window,
        edit_context: &mut EditContext,
        texture_addable: bool,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) -> (Vec<ClippedPrimitive>, TexturesDelta) {
        let raw_input = self.state.take_egui_input(window);

        let full_output = self.context.run(raw_input, |ctx| {
            self.ui(ctx, edit_context, texture_addable, event_proxy);
        });

        self.state
            .handle_platform_output(window, &self.context, full_output.platform_output);

        let clipped_primitives = self.context.tessellate(full_output.shapes);

        (clipped_primitives, full_output.textures_delta)
    }

    pub fn remove_texture(&mut self, index: usize) {
        self.textures.remove(index);
    }

    pub fn reset_textures(&mut self) {
        self.textures.clear();
    }

    fn ui(
        &self,
        ctx: &Context,
        edit_context: &mut EditContext,
        texture_addable: bool,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) {
        let theme = CodeTheme::from_memory(ctx);

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = self.highlighter.highlight(&theme, string);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        let is_dark = ctx.style().visuals.dark_mode;
        TopBottomPanel::bottom("status").show(ctx, |ui| match &self.app_status {
            Some((status, message)) => {
                match status {
                    AppStatus::Info => ui.label(message),
                    AppStatus::Warning => ui.colored_label(
                        if is_dark {
                            Color32::KHAKI
                        } else {
                            Color32::DARK_RED
                        },
                        message,
                    ),
                    AppStatus::Error => ui.colored_label(
                        if is_dark {
                            Color32::LIGHT_RED
                        } else {
                            Color32::DARK_RED
                        },
                        message,
                    ),
                };
            }
            None => {}
        });

        CentralPanel::default().show(ctx, |ui| {
            widgets::global_dark_light_mode_switch(ui);
            ui.horizontal(|ui| {
                if ui.button("Compile").clicked() {
                    event_proxy.send_event(UserEvent::RequestRedraw).unwrap();
                }
                if ui.button("New").clicked() {
                    event_proxy.send_event(UserEvent::NewFile).unwrap();
                }
                if ui.button("Save").clicked() {
                    event_proxy.send_event(UserEvent::SaveFile).unwrap();
                }
                if ui.button("Open").clicked() {
                    event_proxy.send_event(UserEvent::OpenFile).unwrap();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Name: ");
                ui.text_edit_singleline(&mut edit_context.name);
            });

            let image_size = 50.0;
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
                    for (index, texture) in self.textures.iter().enumerate() {
                        if image_upload(ui, image_size, Some(texture))
                            .on_hover_text("Change/Remove (abort selection) the texture")
                            .clicked()
                        {
                            event_proxy
                                .send_event(UserEvent::ChangeTexture(index))
                                .unwrap();
                        }
                    }
                    if texture_addable {
                        if image_upload(ui, image_size, None)
                            .on_hover_text("Add texture")
                            .clicked()
                        {
                            event_proxy.send_event(UserEvent::OpenTexture).unwrap();
                        }
                    }
                });

                ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        let editor = TextEdit::multiline(&mut edit_context.frag)
                            .code_editor()
                            .desired_width(ui.available_width() / 2.0 - 16.0);
                        let font_id = FontSelection::default().resolve(ui.style());
                        let row_height = ui.fonts().row_height(&font_id) as f32;
                        let editor = editor
                            .desired_rows((ui.available_height() / row_height) as usize)
                            .layouter(&mut layouter);
                        editor.show(ui);
                    });
                });
            });
        });
    }
}
