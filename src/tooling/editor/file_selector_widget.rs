use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Context, Margin, Style, Ui};
use std::path::{self, Path, PathBuf};

pub struct FileSelectorWidget {
    path_input: String,
    settings: FileSelectorWidgetSettings,
    navigator: Option<DirNavigator>,
}

pub struct FileSelectorWidgetSettings {
    pub must_exist: bool,
    pub warn_overwrite: bool,
    pub select_text: &'static str,
    pub base_path_is_changeable: bool,
    pub file_extension: &'static str,
}
impl FileSelectorWidgetSettings {
    pub const LOAD: Self = Self {
        must_exist: true,
        warn_overwrite: false,
        select_text: "Load",
        base_path_is_changeable: true,
        file_extension: "ron",
    };
    pub const SAVE: Self = Self {
        must_exist: false,
        warn_overwrite: true,
        select_text: "Save",
        base_path_is_changeable: true,
        file_extension: "ron",
    };
}

struct DirNavigator {
    path: PathBuf,
    parent: Option<PathBuf>,
    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
    selected_file: Option<PathBuf>,
}

impl DirNavigator {
    pub fn read<P: AsRef<Path>>(dir: P) -> Option<Self> {
        let path = dir.as_ref();
        if !path.is_dir() {
            return None;
        }

        let path = match path::absolute(path) {
            Ok(path) => path,
            Err(e) => {
                error!(
                    "Not a valid absolute path: {}. {e:?}",
                    path.to_string_lossy()
                );
                return None;
            }
        };

        if let Ok(entries) = std::fs::read_dir(&path) {
            let mut files = vec![];
            let mut dirs = vec![];
            let entries = entries.flatten().collect::<Vec<_>>();
            for entry in entries.into_iter() {
                let check_path = path.join(entry.file_name());
                if check_path.is_dir() {
                    dirs.push(PathBuf::from(entry.file_name()));
                } else if check_path.is_file() {
                    files.push(PathBuf::from(entry.file_name()));
                } else {
                    error!(
                        "neither file nor dir in list_dir: {}",
                        check_path.to_string_lossy()
                    );
                }
            }
            let parent = path.parent().map(|p| p.to_path_buf());
            Some(DirNavigator {
                path,
                parent,
                files,
                dirs,
                selected_file: None,
            })
        } else {
            None
        }
    }

    pub fn set_selected<P: AsRef<Path>>(&mut self, path: P) {
        let path = self.path.join(path);
        if path.is_file() {
            self.selected_file = Some(path);
        } else {
            self.selected_file = None;
        }
    }
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let mut selected_file_changed = false;
        let mut new_list_dir = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(parent) = &self.parent {
                if ui.button("..").double_clicked() {
                    new_list_dir = DirNavigator::read(&parent);
                }
            }

            for rel in &self.dirs {
                if ui
                    .button(&format!("./{}", rel.to_string_lossy()))
                    .double_clicked()
                {
                    new_list_dir = DirNavigator::read(self.path.join(rel));
                }
            }
            for rel in &self.files {
                if let Ok(abs) = path::absolute(self.path.join(rel)) {
                    let selected = match &self.selected_file {
                        Some(p) => p == abs.as_path(),
                        None => false,
                    };
                    if ui
                        .selectable_label(selected, rel.to_str().unwrap())
                        .clicked()
                    {
                        self.selected_file = Some(abs);
                        selected_file_changed = true;
                    }
                }
            }
        });

        if let Some(new_list_dir) = new_list_dir {
            *self = new_list_dir;
            self.selected_file = None;
            true
        } else {
            selected_file_changed
        }
    }
}

enum ValidFileSelect {
    New(PathBuf),
    Existing(PathBuf),
}
impl ValidFileSelect {
    pub fn from<P: AsRef<Path>>(
        path: P,
        must_exist: bool,
        extension: &'static str,
    ) -> Option<Self> {
        let path = path::absolute(path).ok()?;
        if path.is_dir() {
            return None;
        }
        let path = if let Some(ext) = path.extension() {
            if let Some(ext) = ext.to_str() {
                if ext == extension {
                    path
                } else {
                    path.with_extension(extension)
                }
            } else {
                path.with_extension(extension)
            }
        } else {
            path.with_extension(extension)
        };
        let path = path::absolute(path).ok()?;

        match path.is_file() {
            true => Some(Self::Existing(path)),
            false => {
                if must_exist || path.is_dir() {
                    None
                } else {
                    Some(Self::New(path))
                }
            }
        }
    }
}

pub enum FileSelectorWidgetResult {
    FileSelected(PathBuf),
    CloseRequested,
}

impl FileSelectorWidget {
    pub fn new<P: Into<PathBuf>>(base_path: P, settings: FileSelectorWidgetSettings) -> Self {
        let base_path = base_path.into();
        let navigator = DirNavigator::read(&base_path);
        Self {
            path_input: String::new(),
            settings,
            navigator,
        }
    }
    pub fn show(&mut self, ctx: &mut Context) -> Option<FileSelectorWidgetResult> {
        let mut result = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            let space = ui.available_size();

            egui::Frame::popup(&Style::default())
                .inner_margin(Margin {
                    left: f32::max(0.0, (space.x - 480.0) * 0.5),
                    right: f32::max(0.0, (space.x - 480.0) * 0.5),
                    top: 100.,
                    bottom: 0.,
                })
                // .frame(egui::Frame::dark_canvas(&Style::default()).inner_margin(Margin::same(20.0)))
                .show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("File Selector");

                            let Some(nav) = &mut self.navigator else {
                                ui.colored_label(Color32::RED, "Invalid base path");
                                return;
                            };
                            ui.separator();
                            ui.label(&nav.path.to_string_lossy().to_string());
                            ui.separator();

                            if nav.show(ui) {
                                self.path_input = match &nav.selected_file {
                                    Some(p) => p.file_name().unwrap().to_str().unwrap().to_owned(),
                                    None => String::new(),
                                }
                            }

                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("File Name:");
                                if ui.text_edit_singleline(&mut self.path_input).changed() {
                                    nav.set_selected(&self.path_input);
                                }
                            });

                            ui.separator();

                            ui.horizontal(|ui| {
                                match ValidFileSelect::from(
                                    nav.path.join(&self.path_input),
                                    self.settings.must_exist,
                                    self.settings.file_extension,
                                ) {
                                    Some(ValidFileSelect::Existing(path)) => {
                                        if self.settings.warn_overwrite {
                                            ui.colored_label(
                                                Color32::YELLOW,
                                                "File will be overwritten",
                                            );
                                        }
                                        let btn = ui.add_enabled(
                                            true,
                                            egui::Button::new(&format!(
                                                "{} {}",
                                                self.settings.select_text,
                                                path.file_name().unwrap().to_string_lossy()
                                            )),
                                        );
                                        if btn.clicked() {
                                            result =
                                                Some(FileSelectorWidgetResult::FileSelected(path));
                                        }
                                    }
                                    Some(ValidFileSelect::New(path)) => {
                                        let btn = ui.add_enabled(
                                            true,
                                            egui::Button::new(&format!(
                                                "{} {}",
                                                self.settings.select_text,
                                                path.file_name().unwrap().to_string_lossy()
                                            )),
                                        );
                                        if btn.clicked() {
                                            result =
                                                Some(FileSelectorWidgetResult::FileSelected(path));
                                        }
                                    }
                                    None => {
                                        ui.add_enabled(false, egui::Button::new("Select a file"));
                                    }
                                }
                                if ui.button("Close").clicked() {
                                    result = Some(FileSelectorWidgetResult::CloseRequested);
                                }
                            });
                            ui.separator();
                        });
                    });
                });
        });
        result
    }
}
