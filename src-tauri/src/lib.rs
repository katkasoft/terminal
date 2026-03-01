use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{Emitter, State};

struct TerminalState {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

#[tauri::command]
fn write_to_pty(data: String, state: State<'_, TerminalState>) -> Result<(), String> {
    let mut writer = state.writer.lock().unwrap();
    writer.write_all(data.as_bytes()).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("failed to open pty");
    let shell =  "bash";
    let cmd = CommandBuilder::new(shell);
    pair.slave.spawn_command(cmd).expect("failed to spawn shell");
    let reader = pair.master.try_clone_reader().expect("failed to get reader");
    let writer = pair.master.take_writer().expect("failed to get writer");
    tauri::Builder::default()
        .manage(TerminalState {
            writer: Arc::new(Mutex::new(writer)),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            thread::spawn(move || {
                let mut reader = reader;
                let mut buffer = [0u8; 1024];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                            let _ = handle.emit("write", data);
                        }
                        Err(_) => break,
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![write_to_pty])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}