use anyhow::Result;
use dbus::arg::RefArg;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use dbus::nonblock::SyncConnection;
use dbus::nonblock::{self};
use dbus::Path;
use dbus_crossroads::Crossroads;
use dbus_tokio::connection;
use futures::future;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::{error::Error, time::Duration};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::StreamExt;

use crate::ImportConfigPayload;

// Define a custom error type
#[derive(Debug)]
struct MyError {
    message: String,
}

// Implement the Error trait for the custom error type
impl Error for MyError {}

// Implement the Display trait to provide a human-readable description
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Implement the From trait to convert the custom error type into Box<dyn Error + Send>
impl From<MyError> for Box<dyn Error + Send> {
    fn from(err: MyError) -> Self {
        Box::new(err)
    }
}

pub struct OpenVPN3Dbus {
    connection: Arc<SyncConnection>,
    log_sender: broadcast::Sender<(String, u32, u32, String)>,
    log_receiver: broadcast::Receiver<(String, u32, u32, String)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenVPN3Config {
    pub path: String,
    pub name: String,
    pub used_count: u32,
}

struct Hello {
    called_count: u32,
}

impl OpenVPN3Dbus {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (resource, conn) = connection::new_system_sync()?;

        let _handle = tokio::spawn(async {
            let err = resource.await;
            panic!("Lost connection to D-Bus: {}", err);
        });

        let (tx_log, rx_log) = broadcast::channel::<(String, u32, u32, String)>(16);

        Ok(Self {
            connection: conn,
            log_sender: tx_log,
            log_receiver: rx_log,
        })
    }

    pub fn on_log<F>(&self, callback: F)
    where
        F: Fn(String, u32, u32, String) + Send + 'static,
    {
        let mut rx = self.log_sender.subscribe();
        let cb = Arc::new(Mutex::new(callback));

        tokio::spawn(async move {
            while let Ok((member, group, level, message)) = rx.recv().await {
                cb.lock().await(member, group, level, message);
            }
        });
    }

    async fn server_status(&self, path: String) -> Result<(), Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let handle = tokio::spawn(async move {
            // Create a new crossroads instance.
            // The instance is configured so that introspection and properties interfaces
            // are added by default on object path additions.
            let mut cr = Crossroads::new();

            // Enable async support for the crossroads instance.
            cr.set_async_support(Some((
                conn.clone(),
                Box::new(|x| {
                    tokio::spawn(x);
                }),
            )));

            // Let's build a new interface, which can be used for "Hello" objects.
            let iface_token = cr.register("net.openvpn.v3.backends", |b| {
                b.method_with_cr_async(
                    "Log",
                    ("group", "level", "message"),
                    (),
                    |mut ctx, cr, (group, level, message): (u32, u32, String)| async move {
                        ctx.reply(Ok(()))
                    },
                );

                b.method_with_cr_async(
                    "StatusChange",
                    ("group", "level", "message"),
                    (),
                    |mut ctx, cr, (group, level, message): (u32, u32, String)| async move {
                        ctx.reply(Ok(()))
                    },
                );

                b.method_with_cr_async(
                    "AttentionRequired",
                    ("group", "level", "message"),
                    (),
                    |mut ctx, cr, (group, level, message): (u32, u32, String)| async move {
                        ctx.reply(Ok(()))
                    },
                );
            });

            // Let's add the "/hello" path, which implements the com.example.dbustest interface,
            // to the crossroads instance.
            cr.insert(path, &[iface_token], Hello { called_count: 0 });

            println!("Server is ready to accept method calls.");

            // We add the Crossroads instance to the connection so that incoming method calls will be handled.
            conn.start_receive(
                MatchRule::new_method_call(),
                Box::new(move |msg, conn| {
                    cr.handle_message(msg, conn).unwrap();
                    true
                }),
            );

            future::pending::<()>().await;
            unreachable!()
        });

        Ok(())
    }

    pub async fn signals(&self) -> Result<(), Box<dyn Error + Send>> {
        let conn = self.connection.clone();
        let tx = self.log_sender.clone();

        tokio::spawn(async move {
            let match_rule = MatchRule::new()
                .with_interface("net.openvpn.v3.backends")
                .with_type(dbus::MessageType::Signal);

            let signal_match = conn
                .add_match(match_rule)
                .await
                .map_err(|_| MyError {
                    message: "Failed to add match rule".to_string(),
                })
                .unwrap();

            println!("Listening for signals...");

            let (incoming_signal, mut stream): (
                dbus::nonblock::MsgMatch,
                futures_channel::mpsc::UnboundedReceiver<(_, (u32, u32, String))>,
            ) = signal_match.stream();

            while let Some(v) = stream.next().await {
                println!("Signal received: {:?} {:?}", v.1, v.0);
                let message = v.0;
                let member = message.member().unwrap().into_static();
                let member = member.as_str().unwrap().to_string();

                let (first_code, second_code, message) = v.1;
                let payload = (member.clone(), first_code, second_code, message.clone());

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

    pub async fn remove_config(&self, config_path: String) -> Result<(), Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            dbus::Path::new(config_path).unwrap(),
            Duration::from_secs(2),
            conn,
        );

        proxy
            .method_call("net.openvpn.v3.configuration", "Remove", ())
            .await
            .map_err(|_| MyError {
                message: "Failed to remove config".to_string(),
            })?;

        Ok(())
    }

    pub async fn new_tunnel(&self, config_path: String) -> Result<String, Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            "/net/openvpn/v3/sessions",
            Duration::from_secs(2),
            conn,
        );

        let (session_path,): (Path,) = proxy
            .method_call(
                "net.openvpn.v3.sessions",
                "NewTunnel",
                (dbus::Path::new(config_path).unwrap(),),
            )
            .await
            .map_err(|e| {
                println!("Error: {:?}", e);
                return MyError {
                    message: "Failed to start tunnel".to_string(),
                };
            })?;

        let session_conn = self.connection.clone();
        let proxy_session = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            session_path.clone(),
            Duration::from_secs(2),
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
                        return Err(Box::new(MyError {
                            message: "Failed to start tunnel".to_string(),
                        }));
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

        let server = self
            .server_status(session_path.clone().as_str().unwrap().to_string())
            .await;

        match server {
            Ok(_) => (),
            Err(_) => {
                println!("Failed to start server");
            }
        }

        if let Ok(()) = proxy_session
            .method_call("net.openvpn.v3.sessions", "LogForward", (true,))
            .await
        {
            println!("LogForwarded");
        } else {
            println!("Failed to forward logs");
        }

        if let Ok(data) = proxy_session.get_all("net.openvpn.v3.sessions").await {
            let log_forwards = data.get("log_forwards").unwrap();
            let log_path = log_forwards
                .0
                .as_iter()
                .unwrap()
                .next()
                .unwrap()
                .as_str()
                .unwrap();

            let log_conn = self.connection.clone();
            let log_proxy = nonblock::Proxy::new(
                "net.openvpn.v3.log",
                dbus::Path::new(log_path).unwrap(),
                Duration::from_secs(2),
                log_conn,
            );

            if let Ok(data) = log_proxy.get_all("net.openvpn.v3.log").await {
                println!("LogForwarded: {:?}", data);
            }
        }

        let session_path_as_string = session_path.to_string();

        Ok(session_path_as_string)
    }

    async fn fetch_config_data(
        &self,
        config_path: &str,
    ) -> Result<(String, u32), Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            dbus::Path::new(config_path).unwrap(),
            Duration::from_secs(2),
            conn,
        );

        let config_name: String = proxy
            .get("net.openvpn.v3.configuration", "name")
            .await
            .map_err(|_| MyError {
                message: "Failed to fetch config data".to_string(),
            })?;

        let used_count: u32 = proxy
            .get("net.openvpn.v3.configuration", "used_count")
            .await
            .map_err(|_| MyError {
                message: "Failed to fetch config data".to_string(),
            })?;

        Ok((config_name, used_count))
    }

    pub async fn get_configs(&self) -> Result<Vec<OpenVPN3Config>, Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            "/net/openvpn/v3/configuration",
            Duration::from_secs(2),
            conn,
        );

        let (configs_paths,): (Vec<Path>,) = proxy
            .method_call("net.openvpn.v3.configuration", "FetchAvailableConfigs", ())
            .await
            .map_err(|_| MyError {
                message: "Failed to fetch available configs".to_string(),
            })?;

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
    ) -> Result<String, Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.configuration",
            "/net/openvpn/v3/configuration",
            Duration::from_secs(2),
            conn,
        );

        // read config file from path
        let config_content =
            std::fs::read_to_string(&payload.config_file).map_err(|_| MyError {
                message: "Failed to read config file".to_string(),
            })?;

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
            .map_err(|_| MyError {
                message: "Failed to import config".to_string(),
            })?;

        let as_string = config_path.to_string();

        Ok(as_string)
    }

    pub async fn get_sessions(&self) -> Result<Vec<String>, Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            "/net/openvpn/v3/sessions",
            Duration::from_secs(2),
            conn,
        );

        let (sessions,): (Vec<Path>,) = proxy
            .method_call("net.openvpn.v3.sessions", "FetchAvailableSessions", ())
            .await
            .map_err(|_| MyError {
                message: "Failed to fetch available sessions".to_string(),
            })?;

        let sessions_as_strings = sessions
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        Ok(sessions_as_strings)
    }

    pub async fn disconnect_session(
        &self,
        session_path: String,
    ) -> Result<(), Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            dbus::Path::new(session_path).unwrap(),
            Duration::from_secs(2),
            conn,
        );

        proxy
            .method_call("net.openvpn.v3.sessions", "Disconnect", ())
            .await
            .map_err(|error| {
                println!("Failed to disconnect session: {:?}", error);
                return MyError {
                    message: "Failed to disconnect session".to_string(),
                };
            })?;

        Ok(())
    }

    pub async fn connect_session(&self, session_path: String) -> Result<(), Box<dyn Error + Send>> {
        let conn = self.connection.clone();

        let proxy = nonblock::Proxy::new(
            "net.openvpn.v3.sessions",
            dbus::Path::new(session_path).unwrap(),
            Duration::from_secs(2),
            conn,
        );

        proxy
            .method_call("net.openvpn.v3.sessions", "Connect", ())
            .await
            .map_err(|_| MyError {
                message: "Failed to connect session".to_string(),
            })?;

        Ok(())
    }
}
