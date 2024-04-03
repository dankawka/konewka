use crate::{
    dbus::structs::{OpenVPN3Config, OpenVPN3Session},
    structs::ImportConfigPayload,
    MyState,
};

#[tauri::command]
pub async fn get_openvpn3_configs<'a>(
    state: tauri::State<'a, MyState>,
) -> Result<Vec<OpenVPN3Config>, ()> {
    let configs = match state.openvpn3.get_configs().await {
        Ok(configs) => configs,
        Err(error) => {
            let mut retry = 0;
            loop {
                println!("Failed to get configs, retrying...");
                println!("Error: {:?}", error);
                if retry == 3 {
                    return Ok(vec![]);
                }
                match state.openvpn3.get_configs().await {
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
pub fn select_file() -> Result<String, String> {
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

#[tauri::command]
pub async fn import_openvpn3_config<'a>(
    payload: ImportConfigPayload,
    state: tauri::State<'a, MyState>,
) -> Result<String, String> {
    println!("Importing config: {:?}", payload);
    let path = match state.openvpn3.import_config(payload).await {
        Ok(config_path) => config_path,
        Err(_) => {
            return Ok("Failed to import config".to_string());
        }
    };
    Ok(path)
}

#[tauri::command]
pub async fn get_openvpn3_sessions<'a>(
    state: tauri::State<'a, MyState>,
) -> Result<Vec<OpenVPN3Session>, ()> {
    let sessions = match state.openvpn3.get_sessions().await {
        Ok(configs) => configs,
        Err(error) => {
            let mut retry = 0;
            loop {
                println!("Failed to get sessions, retrying...");
                println!("Error: {:?}", error);
                if retry == 3 {
                    return Ok(vec![]);
                }
                match state.openvpn3.get_sessions().await {
                    Ok(sessions) => break sessions,
                    Err(_) => retry += 1,
                }

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };
    Ok(sessions)
}

#[tauri::command]
pub async fn remove_config<'a>(
    payload: String,
    state: tauri::State<'a, MyState>,
) -> Result<(), ()> {
    match state.openvpn3.remove_config(payload.clone()).await {
        Ok(configs) => configs,
        Err(error) => {
            let mut retry = 0;
            loop {
                println!("Failed to remove config, retrying...");
                println!("Error: {:?}", error);
                if retry == 3 {
                    return Ok(());
                }
                match state.openvpn3.remove_config(payload.clone()).await {
                    Ok(sessions) => break sessions,
                    Err(_) => retry += 1,
                }

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };

    Ok(())
}

#[tauri::command]
pub async fn new_tunnel<'a>(
    payload: String,
    state: tauri::State<'a, MyState>,
) -> Result<String, ()> {
    let session_path = match state.openvpn3.new_tunnel(payload.clone()).await {
        Ok(session_path) => session_path,
        Err(_error) => {
            println!("Failed to create new tunnel");
            return Ok("".to_string());
        }
    };

    Ok(session_path)
}

#[tauri::command]
pub async fn disconnect_session<'a>(
    payload: String,
    state: tauri::State<'a, MyState>,
) -> Result<(), ()> {
    match state.openvpn3.disconnect_session(payload.clone()).await {
        Ok(session_path) => session_path,
        Err(_) => {
            println!("Failed to disconnect session");
            return Ok(());
        }
    };

    Ok(())
}

#[tauri::command]
pub async fn connect_session<'a>(
    payload: String,
    state: tauri::State<'a, MyState>,
) -> Result<(), ()> {
    match state.openvpn3.connect_session(payload.clone()).await {
        Ok(session_path) => session_path,
        Err(_) => {
            println!("Failed to connect session");
            return Ok(());
        }
    };

    Ok(())
}

#[tauri::command]
pub async fn exit_app<'a>(
    state: tauri::State<'a, MyState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    state.openvpn3.disconnect_all().await.unwrap();
    app_handle.exit(0);
    Ok("Exiting".to_string())
}
