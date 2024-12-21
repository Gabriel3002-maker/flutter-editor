use eframe::egui;
use std::process::{Command};
use sysinfo::{System, SystemExt, DiskExt};
use procfs::process::{Process};
use std::io::{self, Read, Write};
use std::fs::{File, read_dir};
use rfd::FileDialog;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Flutter Editor",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    terminal_started: bool,    
    system_info: System,       
    editor_text: String,      
    file_path: Option<String>, 
    folder_path: Option<String>,
    files_in_folder: Vec<String>,
}

impl MyApp {
    fn get_system_info(&mut self) {
        self.system_info.refresh_memory();
        self.system_info.refresh_disks();
    }

    fn get_memory_usage(&self) -> String {
        let total_mem = self.system_info.total_memory();
        let used_mem = self.system_info.used_memory();
        let used_percentage = (used_mem as f64 / total_mem as f64) * 100.0;

        format!(
            "Memoria usada: {} MB / {} MB ({}%)",
            used_mem / 1024,
            total_mem / 1024,
            used_percentage as u8
        )
    }

    fn get_disk_usage(&self) -> String {
        let disks = self.system_info.disks();
        if let Some(disk) = disks.get(0) {
            let total_space = disk.total_space();
            let free_space = disk.available_space();
            let used_percentage = ((total_space - free_space) as f64 / total_space as f64) * 100.0;

            return format!(
                "Disco: {} GB usados / {} GB totales ({}%)",
                (total_space - free_space) / 1024 / 1024 / 1024,
                total_space / 1024 / 1024 / 1024,
                used_percentage as u8
            );
        }
        String::from("No se pudo obtener información del disco.")
    }

    fn get_program_memory_usage(&self) -> String {
        if let Ok(process) = Process::new(std::process::id() as i32) {
            if let Ok(stat) = process.stat() {
                let memory_usage = stat.vsize / 1024 / 1024;
                return format!("Consumo de RAM del programa: {} MB", memory_usage);
            }
        }
        String::from("No se pudo obtener el consumo de RAM del programa.")
    }

    fn open_file(&mut self) -> io::Result<()> {
        if let Some(path) = FileDialog::new().pick_file() {
            let file_path = path.to_str().unwrap().to_string();
            self.file_path = Some(file_path.clone()); 
            let mut file = File::open(file_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            self.editor_text = content; 
        }
        Ok(())
    }

    // Función para guardar el archivo actual
    fn save_file(&mut self) -> io::Result<()> {
        if let Some(file_path) = &self.file_path {
            let mut file = File::create(file_path)?;
            file.write_all(self.editor_text.as_bytes())?;
        } else {
            if let Some(path) = FileDialog::new().save_file() {
                let file_path = path.to_str().unwrap().to_string();
                self.file_path = Some(file_path.clone());
                let mut file = File::create(file_path)?;
                file.write_all(self.editor_text.as_bytes())?;
            }
        }
        Ok(())
    }

    fn open_folder(&mut self) -> io::Result<()> {
        if let Some(path) = FileDialog::new().pick_folder() {
            let folder_path = path.to_str().unwrap().to_string();
            self.folder_path = Some(folder_path.clone());

            if let Ok(entries) = read_dir(folder_path.clone()) {
                let mut files = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(file_name) = entry.file_name().to_str() {
                            files.push(file_name.to_string());
                        }
                    }
                }
                self.files_in_folder = files;
            }
        }
        Ok(())
    }

    fn open_file_from_folder(&mut self, file_name: &str) -> io::Result<()> {
        if let Some(folder_path) = &self.folder_path {
            let file_path = format!("{}/{}", folder_path, file_name);
            let mut file = File::open(file_path.clone())?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            self.editor_text = content; 
            self.file_path = Some(file_path); 
        }
        Ok(())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.get_system_info();

        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .min_height(30.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.button("Archivo");
                    ui.button("Editar");
                    ui.button("Ver");
                    ui.button("Ayuda");

                    if ui.button("Abrir Terminator").clicked() {
                        if !self.terminal_started {
                            launch_terminator().unwrap();
                            self.terminal_started = true;
                        }
                    }
                });
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Explorador de Archivos");

                if ui.button("Abrir archivo").clicked() {
                    self.open_file().unwrap();
                }

                if ui.button("Guardar archivo").clicked() {
                    self.save_file().unwrap();
                }

                if ui.button("Abrir carpeta").clicked() {
                    self.open_folder().unwrap();
                }

                if let Some(folder_path) = &self.folder_path {
                    ui.label(format!("Archivos en: {}", folder_path));
                    
                    let file_names: Vec<String> = self.files_in_folder.clone();

                    for file_name in file_names {
                        if ui.button(&file_name).clicked() {
                            self.open_file_from_folder(&file_name).unwrap();
                        }
                    }
                    
                }
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Terminal");

                ui.label(self.get_program_memory_usage());
                ui.label("La terminal está ejecutándose en una ventana separada.");
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Editor Central de Texto");

            ui.text_edit_multiline(&mut self.editor_text);
        });
    }
}

fn launch_terminator() -> io::Result<()> {
    Command::new("gnome-terminal").spawn()?;
    Ok(())
}
