use std::pin::Pin;

use futures::Future;

use crate::{
    dbus::structs::{OpenVPN3Config, OpenVPN3Session},
    structs::ImportConfigPayload,
    utils, MyState,
};

#[tauri::command]
pub async fn get_openvpn3_configs(
    state: tauri::State<'_, MyState>,
) -> Result<Vec<OpenVPN3Config>, ()> {
    let openvpn3 = state.openvpn3.clone();

    let closure = move || {
        let openvpn3 = openvpn3.clone();
        Box::pin(async move { openvpn3.get_configs().await })
            as Pin<Box<dyn Future<Output = Result<Vec<OpenVPN3Config>, anyhow::Error>> + Send>>
    };

    let configs = match utils::async_retry(closure, 5).await {
        Ok(configs) => configs,
        Err(_) => {
            return Ok(vec![]);
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
    let openvpn3 = state.openvpn3.clone();

    let to_retry = move || {
        let openvpn3 = openvpn3.clone();

        Box::pin(async move { openvpn3.get_sessions().await })
            as Pin<Box<dyn Future<Output = Result<Vec<OpenVPN3Session>, anyhow::Error>> + Send>>
    };

    if let Ok(sessions) = utils::async_retry(to_retry, 3).await {
        return Ok(sessions);
    } else {
        return Err(());
    };
}

#[tauri::command]
pub async fn remove_config(payload: String, state: tauri::State<'_, MyState>) -> Result<(), ()> {
    let openvpn3 = state.openvpn3.clone();

    let to_retry = move || {
        let openvpn3 = openvpn3.clone();
        let payload = payload.clone();

        Box::pin(async move { openvpn3.remove_config(payload.clone()).await })
            as Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>
    };

    if let Ok(_) = utils::async_retry(to_retry, 3).await {
        return Ok(());
    } else {
        return Err(());
    };
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
