// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use dbus::openvpn3::OpenVPN3Dbus;

use commands::{
    connect_session, disconnect_session, exit_app, get_openvpn3_configs, get_openvpn3_sessions,
    import_openvpn3_config, minimize_to_tray, new_tunnel, remove_config, select_file,
};
use structs::LogMessage;
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

mod commands;
mod dbus;
mod structs;
mod utils;

struct MyState {
    openvpn3: Arc<OpenVPN3Dbus>,
}

#[tokio::main]
async fn main() {
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let tray_menu = SystemTrayMenu::new().add_item(show);
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
    let openvpn3_window_events = openvpn3.clone();

    let app = tauri::Builder::default()
        .manage(MyState { openvpn3: openvpn3 })
        .setup(move |app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }

            let main_window = app.get_window("main").unwrap();

            let window = main_window.clone();

            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();

                    let openvpn3_window_events = openvpn3_window_events.clone();
                    let main_window = window.clone();

                    tokio::spawn(async move {
                        if !openvpn3_window_events.has_session().await.unwrap() {
                            main_window.emit("exit_confirmation", false).unwrap();
                        } else {
                            main_window.emit("exit_confirmation", true).unwrap();
                        }
                    });
                }
            });

            openvpn3_logger.on_log(move |path, member, group, level, message| {
                let message = LogMessage {
                    path: path,
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
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_openvpn3_configs,
            select_file,
            import_openvpn3_config,
            get_openvpn3_sessions,
            remove_config,
            new_tunnel,
            disconnect_session,
            connect_session,
            exit_app,
            minimize_to_tray,
        ])
        .build(tauri::generate_context!())
        .unwrap();

    app.run(move |_, _| {});
}
