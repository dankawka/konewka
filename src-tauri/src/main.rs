// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use dbus::openvpn3::{OpenVPN3Config, OpenVPN3Dbus, OpenVPN3Session};

use serde::{Deserialize, Serialize};
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem};
mod dbus;

struct MyState(Arc<OpenVPN3Dbus>);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn get_openvpn3_configs<'a>(
    state: tauri::State<'a, MyState>,
) -> Result<Vec<OpenVPN3Config>, ()> {
    let configs = match state.0.get_configs().await {
        Ok(configs) => configs,
        Err(error) => {
            let mut retry = 0;
            loop {
                println!("Failed to get configs, retrying...");
                println!("Error: {:?}", error);
                if retry == 3 {
                    return Ok(vec![]);
                }
                match state.0.get_configs().await {
                    Ok(configs) => break configs,
                    Err(_) => retry += 1,
                }

                // Wait for 1 second before retrying
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };
    Ok(configs)
}

#[tauri::command]
fn select_file() -> Result<String, String> {
    let file = tauri_api::dialog::select(Some(""), Some(""));
    match file {
        Ok(file) => match file {
            tauri_api::dialog::Response::Okay(file) => {
                return Ok(file);
            }
            _ => {
                return Err("Invalid file selection".to_string());
            }
        },
        Err(_error) => {
            return Ok("".to_string());
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ImportConfigPayload {
    config_name: String,
    single_use: bool,
    persistent: bool,
    config_file: String,
}
#[tauri::command]
async fn import_openvpn3_config<'a>(
    payload: ImportConfigPayload,
    state: tauri::State<'a, MyState>,
) -> Result<String, String> {
    println!("Importing config: {:?}", payload);
    let path = match state.0.import_config(payload).await {
        Ok(config_path) => config_path,
        Err(_) => {
            return Ok("Failed to import config".to_string());
        }
    };
    Ok(path)
}

#[tauri::command]
async fn get_openvpn3_sessions<'a>(
    state: tauri::State<'a, MyState>,
) -> Result<Vec<OpenVPN3Session>, ()> {
    let sessions = match state.0.get_sessions().await {
        Ok(configs) => configs,
        Err(error) => {
            let mut retry = 0;
            loop {
                println!("Failed to get sessions, retrying...");
                println!("Error: {:?}", error);
                if retry == 3 {
                    return Ok(vec![]);
                }
                match state.0.get_sessions().await {
                    Ok(sessions) => break sessions,
                    Err(_) => retry += 1,
                }

                // Wait for 1 second before retrying
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };
    Ok(sessions)
}

#[tauri::command]
async fn remove_config<'a>(payload: String, state: tauri::State<'a, MyState>) -> Result<(), ()> {
    match state.0.remove_config(payload.clone()).await {
        Ok(configs) => configs,
        Err(error) => {
            let mut retry = 0;
            loop {
                println!("Failed to remove config, retrying...");
                println!("Error: {:?}", error);
                if retry == 3 {
                    return Ok(());
                }
                match state.0.remove_config(payload.clone()).await {
                    Ok(sessions) => break sessions,
                    Err(_) => retry += 1,
                }

                // Wait for 1 second before retrying
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };

    Ok(())
}

#[tauri::command]
async fn new_tunnel<'a>(payload: String, state: tauri::State<'a, MyState>) -> Result<String, ()> {
    let session_path = match state.0.new_tunnel(payload.clone()).await {
        Ok(session_path) => session_path,
        Err(_error) => {
            println!("Failed to create new tunnel");
            return Ok("".to_string());
        }
    };

    Ok(session_path)
}

#[tauri::command]
async fn disconnect_session<'a>(
    payload: String,
    state: tauri::State<'a, MyState>,
) -> Result<(), ()> {
    match state.0.disconnect_session(payload.clone()).await {
        Ok(session_path) => session_path,
        Err(_) => {
            println!("Failed to disconnect session");
            return Ok(());
        }
    };

    Ok(())
}

#[tauri::command]
async fn connect_session<'a>(payload: String, state: tauri::State<'a, MyState>) -> Result<(), ()> {
    match state.0.connect_session(payload.clone()).await {
        Ok(session_path) => session_path,
        Err(_) => {
            println!("Failed to connect session");
            return Ok(());
        }
    };

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LogMessage {
    member: String,
    first_flag: u32,
    second_flag: u32,
    message: String,
}

#[tokio::main]
async fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let tray = SystemTray::new().with_menu(tray_menu);

    let openvpn3 = Arc::new(dbus::openvpn3::OpenVPN3Dbus::new().unwrap());

    match openvpn3.signals().await {
        Ok(_) => println!("Successfully connected to OpenVPN3 D-Bus - signals"),
        Err(_) => {
            println!("Failed to connect to OpenVPN3 D-Bus - signals");
            std::process::exit(1);
        }
    }

    let openvpn3_logger = openvpn3.clone();

    tauri::Builder::default()
        .manage(MyState(openvpn3))
        .setup(move |app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }

            let main_window = app.get_window("main").unwrap();

            openvpn3_logger.on_log(move |member, group, level, message| {
                let message = LogMessage {
                    member: member,
                    first_flag: group,
                    second_flag: level,
                    message,
                };
                main_window.emit("log", message).unwrap();
            });

            Ok(())
        })
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![
            get_openvpn3_configs,
            select_file,
            import_openvpn3_config,
            get_openvpn3_sessions,
            remove_config,
            new_tunnel,
            disconnect_session,
            connect_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
