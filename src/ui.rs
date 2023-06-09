mod highlight;
mod image_upload;

use crate::event::{AppStatus, EventProxy, UserEvent};
use egui::{
    style::FontSelection, widgets, Align, Button, CentralPanel, Color32, ColorImage, Context,
    FullOutput, ImageData, Layout, RawInput, ScrollArea, TextEdit, TextureHandle, TextureOptions,
    TopBottomPanel,
};
use highlight::{CodeTheme, Highlighter};
use image_upload::image_upload;

pub struct EditContext {
    pub frag: String,
    pub name: String,
}

pub struct Ui {
    context: Context,
    highlighter: Highlighter,
    textures: Vec<TextureHandle>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            context: Context::default(),
            highlighter: Highlighter::default(),
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
            TextureOptions::LINEAR,
        ));
    }

    pub fn change_texture(&mut self, index: usize, width: u32, height: u32, data: &[u8]) {
        self.textures[index] = self.context.load_texture(
            "debug",
            ImageData::Color(ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                &data,
            )),
            TextureOptions::LINEAR,
        );
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn prepare(
        &mut self,
        raw_input: RawInput,
        edit_context: &mut EditContext,
        event_proxy: &impl EventProxy<UserEvent>,
        state: UiState,
    ) -> FullOutput {
        self.context.run(raw_input, |ctx| {
            self.ui(ctx, edit_context, event_proxy, state);
        })
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
        event_proxy: &impl EventProxy<UserEvent>,
        state: UiState,
    ) {
        let theme = CodeTheme::from_memory(ctx);

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = self.highlighter.highlight(&theme, string);
            layout_job.wrap.max_width = wrap_width;

            ui.fonts(|f| f.layout_job(layout_job))
        };

        let is_dark = ctx.style().visuals.dark_mode;
        TopBottomPanel::bottom("status").show(ctx, |ui| match state.status {
            AppStatus::Info(message) => {
                ui.label(message);
            }
            AppStatus::Warning(message) => {
                ui.colored_label(
                    if is_dark {
                        Color32::KHAKI
                    } else {
                        Color32::DARK_RED
                    },
                    message,
                );
            }
            AppStatus::Error(message) => {
                ui.colored_label(
                    if is_dark {
                        Color32::LIGHT_RED
                    } else {
                        Color32::DARK_RED
                    },
                    message,
                );
            }
            _ => {}
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("About").clicked() {
                    event_proxy.send_event(UserEvent::OpenAbout);
                }

                widgets::global_dark_light_mode_switch(ui);
            });

            ui.horizontal_wrapped(|ui| {
                ui.set_max_width(ui.available_width() / 2.0);

                if ui.button("Compile").clicked() {
                    event_proxy.send_event(UserEvent::RequestRedraw);
                }
                if ui.button("Capture Image").clicked() {
                    event_proxy.send_event(UserEvent::CaptureImage);
                }

                ui.separator();

                if ui.button("New").clicked() {
                    event_proxy.send_event(UserEvent::NewFile);
                }
                if ui.button("Open").clicked() {
                    event_proxy.send_event(UserEvent::OpenFile);
                }

                if ui.button("Save").clicked() {
                    event_proxy.send_event(UserEvent::SaveFile);
                }
                if ui
                    .add_enabled(state.file_saved, Button::new("Save As"))
                    .clicked()
                {
                    event_proxy.send_event(UserEvent::SaveFileAs);
                }

                ui.separator();

                if ui.button("Restart").clicked() {
                    event_proxy.send_event(UserEvent::Restart);
                }
                if ui
                    .button(if state.is_paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    event_proxy.send_event(if state.is_paused {
                        UserEvent::Resume
                    } else {
                        UserEvent::Pause
                    });
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.set_max_width(ui.available_width() / 2.0);

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
                            event_proxy.send_event(UserEvent::ChangeTexture(index));
                        }
                    }
                    if state.texture_addable {
                        if image_upload(ui, image_size, None)
                            .on_hover_text("Add texture")
                            .clicked()
                        {
                            event_proxy.send_event(UserEvent::OpenTexture);
                        }
                    }
                });

                ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        let editor = TextEdit::multiline(&mut edit_context.frag)
                            .code_editor()
                            .desired_width(ui.available_width() / 2.0 - 16.0);

                        let font_id = FontSelection::default().resolve(ui.style());
                        let row_height = self.context.fonts(|fonts| fonts.row_height(&font_id));

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

pub struct UiState {
    pub file_saved: bool,
    pub is_paused: bool,
    pub status: AppStatus,
    pub texture_addable: bool,
}
