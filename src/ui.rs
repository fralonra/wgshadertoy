mod highlight;
mod image_upload;

use crate::{event::UserEvent, viewport::Viewport};
use egui::{
    style::FontSelection, widgets, Align, CentralPanel, ClippedPrimitive, ColorImage, ComboBox,
    Context, ImageData, Layout, ScrollArea, TextEdit, TextureFilter, TextureHandle,
};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit::State;
use highlight::{CodeTheme, Highlighter};
use image_upload::image_upload;
use std::path::PathBuf;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::{
    event::WindowEvent,
    event_loop::{EventLoopProxy, EventLoopWindowTarget},
    window::Window,
};

pub struct EditContext {
    pub frag: String,
    pub name: String,
}

pub struct Ui {
    clipped_primitives: Vec<ClippedPrimitive>,
    context: Context,
    highlighter: Highlighter,
    render_pass: RenderPass,
    screen_descriptor: ScreenDescriptor,
    state: State,
    textures: Vec<TextureHandle>,
}

impl Ui {
    pub fn new(
        device: &Device,
        event_loop: &EventLoopWindowTarget<UserEvent>,
        format: TextureFormat,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        let clipped_primitives = Vec::new();
        let context = egui::Context::default();
        let render_pass = RenderPass::new(device, format, 1);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let mut state = State::new(event_loop);
        state.set_pixels_per_point(scale_factor);

        Self {
            clipped_primitives,
            context,
            highlighter: Highlighter::default(),
            render_pass,
            screen_descriptor,
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

    pub fn draw(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        view: &TextureView,
        viewport: &Viewport,
    ) {
        self.render_pass.update_buffers(
            device,
            queue,
            &self.clipped_primitives,
            &self.screen_descriptor,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Egui Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_viewport(
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
            viewport.min_depth,
            viewport.max_depth,
        );

        self.render_pass
            .execute_with_renderpass(
                &mut render_pass,
                &self.clipped_primitives,
                &self.screen_descriptor,
            )
            .unwrap();
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.state.on_event(&self.context, event);
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        window: &Window,
        edit_context: &mut EditContext,
        shader_frag_path: &Option<PathBuf>,
        presets: &Vec<PathBuf>,
        texture_addable: bool,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |ctx| {
            self.ui(
                ctx,
                edit_context,
                shader_frag_path,
                presets,
                texture_addable,
                event_proxy,
            );
        });

        self.state
            .handle_platform_output(window, &self.context, full_output.platform_output);

        self.clipped_primitives = self.context.tessellate(full_output.shapes);

        self.render_pass
            .add_textures(device, queue, &full_output.textures_delta)
            .unwrap();
    }

    pub fn remove_texture(&mut self, index: usize) {
        self.textures.remove(index);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_descriptor.physical_width = width;
        self.screen_descriptor.physical_height = height;
    }

    pub fn reset_textures(&mut self) {
        self.textures.clear();
    }

    fn ui(
        &self,
        ctx: &Context,
        edit_context: &mut EditContext,
        shader_frag_path: &Option<PathBuf>,
        presets: &Vec<PathBuf>,
        texture_addable: bool,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) {
        let theme = CodeTheme::from_memory(ctx);

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = self.highlighter.highlight(&theme, string);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        CentralPanel::default().show(ctx, |ui| {
            widgets::global_dark_light_mode_switch(ui);
            ui.horizontal(|ui| {
                match shader_frag_path {
                    Some(shader_frag_path) => {
                        ComboBox::from_id_source("fragment")
                            .selected_text(format_preset_label(shader_frag_path))
                            .show_ui(ui, |ui| {
                                for frag in presets {
                                    let mut response = ui.selectable_label(
                                        *shader_frag_path == *frag,
                                        format_preset_label(frag),
                                    );
                                    if response.clicked() {
                                        response.mark_changed();

                                        event_proxy
                                            .send_event(UserEvent::SelectFile(frag.to_path_buf()))
                                            .unwrap();
                                    }
                                }
                            });
                    }
                    None => {
                        ComboBox::from_id_source("fragment")
                            .selected_text(String::default())
                            .show_ui(ui, |ui| {
                                ui.selectable_label(false, String::default());

                                for frag in presets {
                                    let mut response =
                                        ui.selectable_label(false, format_preset_label(frag));
                                    if response.clicked() {
                                        response.mark_changed();

                                        event_proxy
                                            .send_event(UserEvent::SelectFile(frag.to_path_buf()))
                                            .unwrap();
                                    }
                                }
                            });
                    }
                };

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

fn format_preset_label(path: &PathBuf) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .to_str()
        .unwrap_or("untitled")
        .to_owned()
}
