use std::process::Child;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sysinfo::System;

use crate::enums::Page;
use crate::structs::Game;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GameLunch {
    page: Page,
    games: Vec<Game>,

    game: Game,
    location: String,

    status: String,
    launch_status: String,

    #[serde(skip)]
    procs: Vec<Child>,

    pub time: Arc<Mutex<HashMap<String, u64>>>,

    #[serde(skip)]
    thread_spawned: bool,

    removed_values: Vec<String>,
}

impl Default for GameLunch {
    fn default() -> Self {
        Self {
            page: Page::Home,
            games: Vec::new(),
            game: Game {
                name: "".to_string(),
                author: "".to_string(),
                location: "".to_string().into(),
            },

            location: "".to_string(),

            status: "".to_string(),
            launch_status: "".to_string(),

            procs: Vec::new(),

            time: Arc::new(Mutex::new(HashMap::new())),

            thread_spawned: false,

            removed_values: Vec::new(),
        }
    }
}

impl GameLunch {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        // spawn gametime thread

        Default::default()
    }
}

impl eframe::App for GameLunch {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(3)
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // spawn thread on first run
        if !self.thread_spawned {
            // spawn thread

            let time = self.time.clone();

            std::thread::spawn(move || {
                let mut system = System::new_all();

                loop {
                    system.refresh_all();

                    let mut hashmap = time.lock().unwrap();

                    // calculates which processes are running

                    let mut names = vec![];

                    system.processes().iter().for_each(|(_pid, process)| {
                        let name = process.name().to_string_lossy().to_lowercase();

                        if !names.contains(&name) {
                            let val = hashmap.get(&name).unwrap_or(&0) + 5;

                            hashmap.insert(name.clone(), val);

                            names.push(name);
                        }
                    });

                    std::mem::drop(hashmap);

                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            });

            self.thread_spawned = true;
        }
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.page, Page::Home, "Home");
                ui.selectable_value(&mut self.page, Page::Launch, "Launch");
                ui.selectable_value(&mut self.page, Page::AddGame, "Add Game");
                ui.selectable_value(&mut self.page, Page::ProcTime, "Process Time");
                ui.selectable_value(&mut self.page, Page::Settings, "Settings");

                if ui.button("Close Launcher").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("PANIC").clicked() {
                    // kill all subprocesses

                    for proc in &mut self.procs {
                        println!("Killing {:?}", proc);
                        let _ = proc.kill(); // this unwrap doesnt matter
                    }

                    #[cfg(unix)]
                    {
                        // kill all processes on linux only

                        for game in &self.games {
                            let loc = game
                                .location
                                .to_string_lossy()
                                .split('/')
                                .last()
                                .unwrap_or("")
                                .to_string();

                            std::process::Command::new("sh")
                                .arg("-c")
                                .arg(format!("kill $(pidof {})", loc))
                                .status()
                                .unwrap_or_default();
                        }
                    }

                    #[cfg(not(unix))]
                    {
                        // kill all processes on windows

                        for game in &self.games {
                            for game in &self.games {
                                let loc = game
                                    .location
                                    .to_string_lossy()
                                    .split('\\')
                                    .last()
                                    .unwrap_or("")
                                    .to_string();

                                std::process::Command::new("cmd")
                                    .arg("/C")
                                    .arg(format!("taskkill $(pidof {})", loc))
                                    .status()
                                    .unwrap_or_default();
                            }
                        }
                    }

                    std::process::exit(0);
                }

                ui.label("GameLunch v0.1.0 by Aityz");
            });
        });

        if self.page == Page::ProcTime {
            egui::SidePanel::left("left").show(ctx, |ui| {
                let mut i = 0;

                ui.heading("Hidden Processes");

                if ui.button("Sort").clicked() {
                    self.removed_values.sort();
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for val in self.removed_values.clone() {
                        ui.horizontal(|ui| {
                            ui.label(val);

                            if ui.button("Restore").clicked() {
                                self.removed_values.remove(i);
                            }
                        });

                        i += 1;
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.page {
            Page::Home => {
                ui.vertical_centered(|ui| {
                    ui.heading("Welcome to GameLunch");
                    ui.label("GameLunch is a game launcher for games that may be in scattered folders, and is made to organize your game library. It is like steam, but without the marketplace");

                    ui.heading("Features");

                    ui.label("- Panic button, isntantly kill all games and the launcher");
                    ui.label("- Fast, as it is written in Rust");
                    ui.label("- Track time on all games");
                    ui.label("- Supports all non-DRM games (you must crack DRM games to use them here)");
                    ui.label("- Works on Linux, macOS and Windows");

                    if ui.button("Get Started").clicked() {
                        self.page = Page::AddGame;
                    }
                });
            }

            Page::Launch => {
                ui.vertical_centered(|ui| {
                    ui.heading("Launch Game");
                });

                let mut i = 0;

                // get game time data

                let data = self.time.lock().unwrap();

                for game in self.games.clone() { // data is cloned to save borrow checker
                    ui.horizontal(|ui| {

                        // get the data

                        let mut sep = "/";

                        #[cfg(not(unix))]
                        {
                            sep = "\\";
                        }

                        let time = game.location.to_string_lossy().split(sep).last().unwrap_or_default().to_lowercase();

                        ui.label(format!("{} by {}, {}", game.name, game.author, format_time(data.get(&time).unwrap_or(&0))));
                        if ui.button("Launch").clicked() {
                            let proc = std::process::Command::new(&game.location).spawn();

                            if let Ok(proc) = proc {
                                self.procs.push(proc);

                                self.launch_status = "Launched game".to_string();
                            } else {
                                self.launch_status = "Failed to launch game".to_string();
                            }
                        }
                        if ui .button("Remove").clicked() {
                            let _ = self.games.remove(i);
                        }

                        i += 1;
                    });
                }

                ui.separator();

                ui.label(&self.launch_status);
            }

            Page::AddGame => {
                ui.heading("Add Game");

                ui.horizontal(|ui| {
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut self.game.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Author: ");
                    ui.text_edit_singleline(&mut self.game.author);
                });

                ui.horizontal(|ui| {
                    ui.label("Location: ");
                    ui.text_edit_singleline(&mut self.location);
                });

                if ui.button("Add Game").clicked() {
                    // do some calculating

                    let path = std::path::PathBuf::from(&self.location);

                    if !path.exists() {
                        self.status = "Game does not exist".to_string();
                    } else if self.game.name.is_empty() {
                        self.status = "Game requires a name".to_string();
                    } else if self.game.author.is_empty() {
                        self.status = "Game requires an author".to_string();
                    } else {
                        self.games.push(Game {
                            name: self.game.name.clone(),
                            author: self.game.author.clone(),
                            location: path
                        });

                        self.game.author = "".to_string();
                        self.game.name = "".to_string();
                        self.location = "".to_string();

                        self.status = "".to_string();
                    }
                }

                ui.label(&self.status);
            }

            Page::ProcTime => {
                ui.vertical_centered(|ui| {
                    ui.heading("Process Time");
                });

                let data = self.time.lock().unwrap();

                if ui.button("Hide All").clicked() {
                    for (key, _value) in data.iter() {
                        self.removed_values.push(key.to_string());

                        self.removed_values.dedup();
                    }
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (key, value) in data.iter() {
                        if !self.removed_values.contains(key) {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}, {}", key, format_time(value)));

                                if ui.button("Hide").clicked() {
                                    self.removed_values.push(key.to_string());

                                    self.removed_values.dedup();
                                }
                            });
                        }
                    }
                });
            }

            _ => {}

        });
    }
}

fn format_time(time: &u64) -> String {
    if *time < 60 {
        format!("{} seconds", time)
    } else if *time < 3600 {
        format!("{} minutes", time / 60)
    } else {
        format!("{} hours", time / 3600)
    }
}
