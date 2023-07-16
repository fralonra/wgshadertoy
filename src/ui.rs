mod highlight;
mod image_upload;
mod utils;

use crate::{
    event::{AppStatus, EventProxy, UserEvent},
    shortcut::Shortcut,
};
use egui::{
    menu, style::FontSelection, widgets, Align, Button, CentralPanel, Color32, ColorImage, Context,
    FontData, FontDefinitions, FontFamily, FullOutput, ImageData, Layout, RawInput, ScrollArea,
    TextEdit, TextureHandle, TextureOptions, TopBottomPanel,
};
use highlight::{CodeTheme, Highlighter};
use image_upload::ImageUpload;
use material_icons::{icon_to_char, Icon};

pub struct EditContext {
    pub frag: String,
    pub name: String,
}

pub struct Ui {
    context: Context,
    highlighter: Highlighter,
    shortcut: Shortcut,
    textures: Vec<TextureHandle>,
}

impl Ui {
    pub fn new() -> Self {
        let mut context = Context::default();

        setup_fonts(&mut context);

        Self {
            context,
            highlighter: Highlighter::default(),
            shortcut: Shortcut::new(),
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
        if ctx.input_mut(|i| i.consume_shortcut(&self.shortcut.app_quit)) {
            event_proxy.send_event(UserEvent::Quit);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.shortcut.file_new)) {
            event_proxy.send_event(UserEvent::NewFile);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.shortcut.file_open)) {
            event_proxy.send_event(UserEvent::OpenFile);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.shortcut.file_save)) {
            event_proxy.send_event(UserEvent::SaveFile);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.shortcut.file_save_as)) {
            event_proxy.send_event(UserEvent::SaveFileAs);
        }

        let theme = CodeTheme::from_memory(ctx);

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = self.highlighter.highlight(&theme, string);
            layout_job.wrap.max_width = wrap_width;

            ui.fonts(|f| f.layout_job(layout_job))
        };

        let is_dark = ctx.style().visuals.dark_mode;

        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(
                            Button::new("New")
                                .shortcut_text(ui.ctx().format_shortcut(&self.shortcut.file_new)),
                        )
                        .clicked()
                    {
                        event_proxy.send_event(UserEvent::NewFile);

                        ui.close_menu();
                    }

                    if ui
                        .add(
                            Button::new("Open")
                                .shortcut_text(ui.ctx().format_shortcut(&self.shortcut.file_open)),
                        )
                        .clicked()
                    {
                        event_proxy.send_event(UserEvent::OpenFile);

                        ui.close_menu();
                    }

                    ui.separator();

                    if ui
                        .add(
                            Button::new("Save")
                                .shortcut_text(ui.ctx().format_shortcut(&self.shortcut.file_save)),
                        )
                        .clicked()
                    {
                        event_proxy.send_event(UserEvent::SaveFile);

                        ui.close_menu();
                    }

                    if ui
                        .add(
                            Button::new("Save As").shortcut_text(
                                ui.ctx().format_shortcut(&self.shortcut.file_save_as),
                            ),
                        )
                        .clicked()
                    {
                        event_proxy.send_event(UserEvent::SaveFileAs);

                        ui.close_menu();
                    }

                    ui.separator();

                    if ui
                        .add(
                            Button::new("Quit")
                                .shortcut_text(ui.ctx().format_shortcut(&self.shortcut.app_quit)),
                        )
                        .clicked()
                    {
                        event_proxy.send_event(UserEvent::Quit);

                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        event_proxy.send_event(UserEvent::OpenAbout);

                        ui.close_menu();
                    }
                });
            });
        });

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
                ui.set_max_width(ui.available_width() / 2.0);

                if ui
                    .button(icon_to_char(Icon::PlayArrow).to_string())
                    .on_hover_text("Compile and run")
                    .clicked()
                {
                    event_proxy.send_event(UserEvent::RequestRedraw);
                }
                if ui
                    .button(icon_to_char(Icon::ScreenshotMonitor).to_string())
                    .on_hover_text("Capture image")
                    .clicked()
                {
                    event_proxy.send_event(UserEvent::CaptureImage);
                }

                ui.separator();

                if ui
                    .button(icon_to_char(Icon::PlayCircleFilled).to_string())
                    .on_hover_text("Restart")
                    .clicked()
                {
                    event_proxy.send_event(UserEvent::Restart);
                }
                if ui
                    .button(
                        icon_to_char(if state.is_paused {
                            Icon::ReplayCircleFilled
                        } else {
                            Icon::PauseCircleFilled
                        })
                        .to_string(),
                    )
                    .on_hover_text(if state.is_paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    event_proxy.send_event(if state.is_paused {
                        UserEvent::Resume
                    } else {
                        UserEvent::Pause
                    });
                }

                ui.separator();

                widgets::global_dark_light_mode_buttons(ui);
            });

            ui.horizontal_wrapped(|ui| {
                ui.set_max_width(ui.available_width() / 2.0);

                ui.label("Name: ");
                ui.text_edit_singleline(&mut edit_context.name);
            });

            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                ui.with_layout(Layout::left_to_right(Align::BOTTOM), |ui| {
                    for (index, texture) in self.textures.iter().enumerate() {
                        ui.add(
                            ImageUpload::new(Some(texture.id()))
                                .edit_hint("Change texture")
                                .remove_hint("Remove texture")
                                .on_edit(|| {
                                    event_proxy.send_event(UserEvent::ChangeTexture(index));
                                })
                                .on_remove(|| {
                                    event_proxy.send_event(UserEvent::RemoveTexture(index));
                                }),
                        );
                    }

                    if state.texture_addable {
                        let resp = ui.add(ImageUpload::new(None)).on_hover_text("Add texture");

                        if resp.clicked() {
                            event_proxy.send_event(UserEvent::OpenTexture);
                        }
                    }
                });

                ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
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

fn setup_fonts(ctx: &mut Context) {
    let mut fonts = FontDefinitions::default();

    const FONT_MATERIAL_ICON: &'static str = "MaterialIcons-Regular";

    fonts.font_data.insert(
        FONT_MATERIAL_ICON.to_owned(),
        FontData::from_static(material_icons::FONT),
    );

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push(FONT_MATERIAL_ICON.to_owned());
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push(FONT_MATERIAL_ICON.to_owned());
    }

    ctx.set_fonts(fonts);
}
