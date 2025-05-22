use eframe::{egui, NativeOptions};
use std::{
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

#[derive(Default)]
struct AppState {
    server_ip: String,
    port: u16,
    username: String,
    password: String,
    socks_port: u16,
    status: String,
    connecting: bool,
    child_process: Option<Child>,
}

struct AppWrapper {
    state: Arc<MutexAppState>,
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SSH SOCKS5 GUI");

            ui.horizontal(|ui| {
                ui.label("Server IP:");
                ui.text_edit_singleline(&mut state.server_ip);
            });

            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(egui::DragValue::new(&mut state.port).clamp_range(1..=65535));
            });

            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.text_edit_singleline(&mut state.username);
            });

            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut state.password).password(true));
            });

            ui.horizontal(|ui| {
                ui.label("SOCKS Port:");
                ui.add(egui::DragValue::new(&mut state.socks_port).clamp_range(1..=65535));
            });

            if ui
                .add_enabled(
                    !state.connecting && state.child_process.is_none(),
                    egui::Button::new("Connect"),
                )
                .clicked()
            {
                let state_clone = self.state.clone();
                state.status = "Connecting...".to_string();
                state.connecting = true;

                thread::spawn(move || {
                    let mut state = state_clone.lock().unwrap();

                    let ip = if state.server_ip.trim().is_empty() {
                        "192.168.1.1".to_string()
                    } else {
                        state.server_ip.clone()
                    };

                    let username = if state.username.trim().is_empty() {
                        "user".to_string()
                    } else {
                        state.username.clone()
                    };

                    let password = if state.password.trim().is_empty() {
                        "user@123".to_string()
                    } else {
                        state.password.clone()
                    };

                    let port = if state.port == 0 { 22 } else { state.port };
                    let socks_port = if state.socks_port == 0 {
                        1080
                    } else {
                        state.socks_port
                    };

                    let mut child = match Command::new("sshpass")
                        .arg("-p")
                        .arg(password)
                        .arg("ssh")
                        .arg("-D")
                        .arg(socks_port.to_string())
                        .arg("-p")
                        .arg(port.to_string())
                        .arg("-o")
                        .arg("StrictHostKeyChecking=no")
                        .arg(format!("{}@{}", username, ip))
                        .arg("-N")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                    {
                        Ok(child) => child,
                        Err(e) => {
                            state.status = format!("Failed: {}", e);
                            state.connecting = false;
                            return;
                        }
                    };

                    state.status = "CONNECTED".to_string();
                    state.child_process = Some(child);
                    state.connecting = false;
                });
            }

            if ui
                .add_enabled(
                    state.child_process.is_some(),
                    egui::Button::new("Disconnect"),
                )
                .clicked()
            {
                if let Some(mut child) = state.child_process.take() {
                    let _ = child.kill();
                    state.status = "DISCONNECTED".to_string();
                }
            }

            ui.separator();
            ui.label(format!("Status: {}", state.status));
        });

        ctx.request_repaint(); // 强制 UI 刷新
    }
}

fn main() -> Result<(), eframe::Error> {
    let app = AppWrapper {
        state: Arc::new(Mutex::new(AppState {
            server_ip: "192.168.1.1".to_string(),
            port: 22,
            username: "user".to_string(),
            password: "user@123".to_string(),
            socks_port: 1080,
            status: "Idle".to_string(),
            ..Default::default()
        })),
    };

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SSH SOCKS5 GUI",
        native_options,
        Box::new(|_cc| Box::new(app)),
    )
}
