use anyhow::{Context, Result};
use dbus::arg::RefArg;
use dbus::message::MatchRule;
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use dbus::nonblock::{self, SyncConnection};
use dbus::Path;
use dbus_tokio::connection;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tokio_stream::StreamExt;

use crate::structs::ImportConfigPayload;

use super::structs::{OpenVPN3Config, OpenVPN3Session};

pub struct OpenVPN3Dbus {
    connection: Arc<SyncConnection>,
    log_sender: broadcast::Sender<(String, String, u32, u32, String)>,
}

impl OpenVPN3Dbus {
    pub fn new() -> Result<Self, anyhow::Error> {
        let (resource, conn) = connection::new_system_sync()?;

        let _handle = tokio::spawn(async {
            let err = resource.await;
            panic!("Lost connection to D-Bus: {}", err);
        });

        let (tx_log, _) = broadcast::channel::<(String, String, u32, u32, String)>(16);

        Ok(Self {
            connection: conn,
            log_sender: tx_log,
        })
    }

    pub fn on_log<F>(&self, callback: F)
    where
        F: Fn(String, String, u32, u32, String) + Send + 'static,
    {
        let mut rx = self.log_sender.subscribe();
        let cb = Arc::new(Mutex::new(callback));

        tokio::spawn(async move {
            while let Ok((path, member, group, level, message)) = rx.recv().await {
                cb.lock().await(path, member, group, level, message);
            }
        });
    }

    pub async fn signals(&self) -> Result<(), anyhow::Error> {
        let conn = self.connection.clone();
        let tx = self.log_sender.clone();

        tokio::spawn(async move {
            let match_rule = MatchRule::new()
                .with_interface("net.openvpn.v3.backends")
                .with_type(dbus::MessageType::Signal);

            let signal_match = conn.add_match(match_rule).await.unwrap();

            println!("Listening for signals...");

            let (_incoming_signal, mut stream): (
                dbus::nonblock::MsgMatch,
                futures_channel::mpsc::UnboundedReceiver<(_, (u32, u32, String))>,
            ) = signal_match.stream();

            while let Some(v) = stream.next().await {
                let message = v.0;
                let member = message.member().unwrap().into_static();
                let member = member.as_str().unwrap().to_string();

                let path = message.path().unwrap().into_static();

                let (first_code, second_code, message) = v.1;
                let payload = (
                    path.to_string(),
                    member.clone(),
                    first_code,
                    second_code,
                    message.clone(),
                );

                let cmember = member.clone();
                if cmember == "StatusChange" && first_code == 3 && second_code == 22 {
                    match open::that(message) {
                        Ok(_) => (),
                        Err(_) => {
                            println!("Failed to open link");
                        }
                    }
                }

                match tx.send(payload) {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Failed to send signal");
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn remove_config(&self, config_path: String) -> Result<(), anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            dbus::Path::new(config_path).unwrap(),
            Duration::from_secs(5),
            conn,
        );

        proxy
            .method_call("net.openvpn.v3.configuration", "Remove", ())
            .await
            .with_context(|| "Failed to remove config")?;

        Ok(())
    }

    pub async fn new_tunnel(&self, config_path: String) -> Result<String, anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            "/net/openvpn/v3/sessions",
            Duration::from_secs(5),
            conn,
        );

        let (session_path,): (Path,) = proxy
            .method_call(
                "net.openvpn.v3.sessions",
                "NewTunnel",
                (dbus::Path::new(config_path).unwrap(),),
            )
            .await
            .with_context(|| "Failed to create new tunnel")?;

        let session_conn = self.connection.clone();
        let proxy_session = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            session_path.clone(),
            Duration::from_secs(5),
            session_conn,
        );

        let result: Result<(), dbus::Error> = proxy
            .method_call("net.openvpn.v3.sessions", "Ready", ())
            .await;

        let _ = match result {
            Err(_) => {
                let mut retry = 0;
                loop {
                    println!("Failed to ready session, retrying...");
                    if retry == 3 {
                        return Err(anyhow::Error::msg("Failed to ready session"));
                    }
                    match proxy_session
                        .method_call("net.openvpn.v3.sessions", "Ready", ())
                        .await
                    {
                        Ok(()) => {
                            println!("Session ready");
                            break;
                        }
                        Err(_) => retry += 1,
                    };

                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
            _ => (),
        };

        if let Ok(()) = proxy_session
            .method_call("net.openvpn.v3.sessions", "LogForward", (true,))
            .await
        {
            println!("LogForwarded");
        } else {
            println!("Failed to forward logs");
        }

        let session_path_as_string = session_path.to_string();

        Ok(session_path_as_string)
    }

    async fn fetch_config_data(&self, config_path: &str) -> Result<(String, u32), anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            dbus::Path::new(config_path).unwrap(),
            Duration::from_secs(5),
            conn,
        );

        let config_name: String = proxy
            .get("net.openvpn.v3.configuration", "name")
            .await
            .with_context(|| "Failed to fetch config data")?;

        let used_count: u32 = proxy
            .get("net.openvpn.v3.configuration", "used_count")
            .await
            .with_context(|| "Failed to fetch config data")?;

        Ok((config_name, used_count))
    }

    pub async fn get_configs(&self) -> Result<Vec<OpenVPN3Config>, anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            "/net/openvpn/v3/configuration",
            Duration::from_secs(5),
            conn,
        );

        let (configs_paths,): (Vec<Path>,) = proxy
            .method_call("net.openvpn.v3.configuration", "FetchAvailableConfigs", ())
            .await
            .with_context(|| "Failed to fetch available configs")?;

        let mut configs = vec![];
        for config in configs_paths.iter() {
            let config_data = self.fetch_config_data(config).await?;

            configs.push(OpenVPN3Config {
                path: config.to_string(),
                name: config_data.0,
                used_count: config_data.1,
            })
        }

        Ok(configs)
    }

    pub async fn import_config(
        &self,
        payload: ImportConfigPayload,
    ) -> Result<String, anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            "/net/openvpn/v3/configuration",
            Duration::from_secs(5),
            conn,
        );

        // read config file from path
        let config_content =
            std::fs::read_to_string(&payload.config_file).with_context(|| "Failed to read file")?;

        let (config_path,): (dbus::Path,) = proxy
            .method_call(
                "net.openvpn.v3.configuration",
                "Import",
                (
                    payload.config_name,
                    config_content,
                    payload.single_use,
                    payload.persistent,
                ),
            )
            .await
            .with_context(|| "Failed to import config")?;

        let as_string = config_path.to_string();

        Ok(as_string)
    }

    pub async fn get_sessions(&self) -> Result<Vec<OpenVPN3Session>, anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            "/net/openvpn/v3/sessions",
            Duration::from_secs(5),
            conn,
        );

        let (sessions,): (Vec<Path>,) = proxy
            .method_call("net.openvpn.v3.sessions", "FetchAvailableSessions", ())
            .await
            .with_context(|| "Failed to fetch available sessions")?;

        let mut sessions_with_data: Vec<OpenVPN3Session> = vec![];

        for session in sessions.iter() {
            let session_conn = self.connection.clone();
            let session_proxy = nonblock::Proxy::new(
                "net.openvpn.v3.sessions",
                session.clone(),
                Duration::from_secs(5),
                session_conn,
            );

            let (major_code, minor_code, status_message): (u32, u32, String) = session_proxy
                .get("net.openvpn.v3.sessions", "status")
                .await
                .with_context(|| "Failed to fetch session data")?;

            let session_created: u64 = session_proxy
                .get("net.openvpn.v3.sessions", "session_created")
                .await
                .with_context(|| "Failed to fetch session data")?;

            let session = OpenVPN3Session {
                path: session.to_string(),
                major_code,
                minor_code,
                status_message,
                session_created,
            };

            sessions_with_data.push(session);
        }

        Ok(sessions_with_data)
    }

    pub async fn has_session(&self) -> Result<bool, anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            "/net/openvpn/v3/sessions",
            Duration::from_secs(5),
            conn,
        );

        let (sessions,): (Vec<Path>,) = proxy
            .method_call("net.openvpn.v3.sessions", "FetchAvailableSessions", ())
            .await
            .with_context(|| "Failed to fetch available sessions")?;

        if sessions.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn disconnect_session(&self, session_path: String) -> Result<(), anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            dbus::Path::new(session_path).unwrap(),
            Duration::from_secs(5),
            conn,
        );

        proxy
            .method_call("net.openvpn.v3.sessions", "Disconnect", ())
            .await
            .with_context(|| "Failed to disconnect session")?;

        Ok(())
    }

    pub async fn connect_session(&self, session_path: String) -> Result<(), anyhow::Error> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            dbus::Path::new(session_path).unwrap(),
            Duration::from_secs(5),
            conn,
        );

        proxy
            .method_call("net.openvpn.v3.sessions", "Connect", ())
            .await
            .with_context(|| "Failed to connect session")?;

        Ok(())
    }

    pub async fn disconnect_all(&self) -> Result<(), anyhow::Error> {
        let sessions = self.get_sessions().await?;

        for session in sessions.iter() {
            self.disconnect_session(session.path.clone()).await?;
        }

        Ok(())
    }
}
