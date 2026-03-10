use crate::ascii::{AsciiArt, generate_ascii_art_with_aspect};
use eframe::egui;
use image::DynamicImage;
use std::sync::Arc;

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "ASCII Renderer GUI",
        options,
        Box::new(|_cc| Ok(Box::<AsciiApp>::default())),
    )
}

struct AsciiApp {
    image: Option<Arc<DynamicImage>>,
    ascii_art: Option<AsciiArt>,
    cached_job: Option<egui::text::LayoutJob>,
    path: String,
    font_size: f32,
    char_aspect: f32,

    receiver: Option<std::sync::mpsc::Receiver<(AsciiArt, egui::text::LayoutJob)>>,
    is_generating: bool,
    target_size: Option<(u32, u32)>,
}

impl Default for AsciiApp {
    fn default() -> Self {
        Self {
            image: None,
            ascii_art: None,
            cached_job: None,
            path: "cover.png".to_owned(),
            font_size: 8.0,
            char_aspect: 0.5,

            receiver: None,
            is_generating: false,
            target_size: None,
        }
    }
}

impl eframe::App for AsciiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.receiver {
            if let Ok((art, job)) = rx.try_recv() {
                self.ascii_art = Some(art);
                self.cached_job = Some(job);
                self.is_generating = false;
                self.receiver = None;
            }
        }

        let mut force_update = false;

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Image Path:");
                ui.text_edit_singleline(&mut self.path);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(
                            "Images",
                            &["png", "jpg", "jpeg", "webp", "bmp", "ico", "tiff"],
                        )
                        .pick_file()
                    {
                        self.path = path.display().to_string();
                    }
                }
                if ui.button("Load").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(bytes) = std::fs::read(&self.path) {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            self.image = Some(Arc::new(img));
                            self.ascii_art = None;
                            self.cached_job = None;
                            self.target_size = None;
                            force_update = true;
                        }
                    }
                }

                ui.separator();

                let mut changed = false;
                ui.label("Font Size:");
                if ui
                    .add(egui::Slider::new(&mut self.font_size, 2.0..=24.0).text("px"))
                    .changed()
                {
                    changed = true;
                }

                ui.label("Aspect Ratio:");
                if ui
                    .add(egui::Slider::new(&mut self.char_aspect, 0.2..=1.5))
                    .changed()
                {
                    changed = true;
                }

                if changed {
                    self.target_size = None;
                    force_update = true;
                }

                if self.is_generating {
                    ui.separator();
                    ui.spinner();
                    ui.label("Generating...");
                }
            });
            ui.add_space(8.0);
        });

        let frame = egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::BLACK);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            if let Some(img) = &self.image {
                let char_height = self.font_size;
                let char_width = self.font_size * self.char_aspect;

                let available_size = ui.available_size();
                let max_w = (available_size.x / char_width).max(1.0) as u32;
                let max_h = (available_size.y / char_height).max(1.0) as u32;

                let needs_update = match self.target_size {
                    Some((w, h)) => w != max_w || h != max_h,
                    None => true,
                };

                if (force_update || needs_update) && max_w > 0 && max_h > 0 {
                    self.target_size = Some((max_w, max_h));
                    self.is_generating = true;

                    let (tx, rx) = std::sync::mpsc::channel();
                    self.receiver = Some(rx);

                    let img_clone = Arc::clone(img);
                    let font_size = self.font_size;
                    let char_aspect = self.char_aspect;
                    let max_w = max_w;
                    let max_h = max_h;
                    let ctx_clone = ctx.clone();

                    std::thread::spawn(move || {
                        let art =
                            generate_ascii_art_with_aspect(&img_clone, max_w, max_h, char_aspect);

                        let mut job = egui::text::LayoutJob::default();

                        // Disable wrapping for performance
                        job.wrap = egui::text::TextWrapping {
                            max_width: f32::INFINITY,
                            max_rows: usize::MAX,
                            break_anywhere: false,
                            overflow_character: None,
                        };

                        let font_id = egui::FontId::monospace(font_size);

                        for y in 0..art.height {
                            let mut current_color = None;
                            let mut current_text = String::new();

                            for x in 0..art.width {
                                let idx = (y * art.width + x) as usize;
                                if idx < art.characters.len() {
                                    let c = &art.characters[idx];

                                    let quantize = 16;
                                    let r = ((c.r as u16 + quantize / 2) / quantize * quantize)
                                        .min(255) as u8;
                                    let g = ((c.g as u16 + quantize / 2) / quantize * quantize)
                                        .min(255) as u8;
                                    let b = ((c.b as u16 + quantize / 2) / quantize * quantize)
                                        .min(255) as u8;

                                    let color = egui::Color32::from_rgb(r, g, b);

                                    if Some(color) == current_color {
                                        current_text.push(c.character);
                                    } else {
                                        if !current_text.is_empty() {
                                            if let Some(cc) = current_color {
                                                job.append(
                                                    &current_text,
                                                    0.0,
                                                    egui::text::TextFormat {
                                                        font_id: font_id.clone(),
                                                        color: cc,
                                                        line_height: Some(font_size),
                                                        ..Default::default()
                                                    },
                                                );
                                            }
                                            current_text.clear();
                                        }
                                        current_color = Some(color);
                                        current_text.push(c.character);
                                    }
                                }
                            }

                            if !current_text.is_empty() {
                                if let Some(cc) = current_color {
                                    job.append(
                                        &current_text,
                                        0.0,
                                        egui::text::TextFormat {
                                            font_id: font_id.clone(),
                                            color: cc,
                                            line_height: Some(font_size),
                                            ..Default::default()
                                        },
                                    );
                                }
                            }

                            job.append(
                                "\n",
                                0.0,
                                egui::text::TextFormat {
                                    font_id: font_id.clone(),
                                    line_height: Some(font_size),
                                    ..Default::default()
                                },
                            );
                        }

                        let _ = tx.send((art, job));
                        ctx_clone.request_repaint();
                    });
                }

                if let Some(job) = &self.cached_job {
                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.add(egui::Label::new(job.clone()));
                    });
                } else if self.is_generating {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("Generating...").color(egui::Color32::WHITE));
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Enter an image path and click Load.")
                            .color(egui::Color32::WHITE),
                    );
                });
            }
        });
    }
}
