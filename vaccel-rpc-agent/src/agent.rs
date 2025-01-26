// SPDX-License-Identifier: Apache-2.0

use crate::{AgentService, Error, Result};
use std::{
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
};
#[cfg(feature = "async")]
use ttrpc::asynchronous::Server;
#[cfg(not(feature = "async"))]
use ttrpc::sync::Server;
use vaccel::Config as VaccelConfig;
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::create_agent_service;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::create_agent_service;

#[derive(Default)]
pub struct Agent {
    pub server_address: String,
    vaccel_config: Arc<Mutex<Option<VaccelConfig>>>,
    server: Option<Server>,
}

impl Agent {
    pub fn new(server_address: &str) -> Self {
        Self {
            server_address: server_address.to_string(),
            ..Default::default()
        }
    }

    pub fn set_server_address(&mut self, address: &str) -> Result<&mut Self> {
        if self.server.is_some() {
            return Err(Error::AgentError(
                "Cannot change server_address of initialized agent".to_string(),
            ));
        }
        self.server_address = address.to_string();
        Ok(self)
    }

    pub fn set_vaccel_config(&self, config: VaccelConfig) -> Result<&Self> {
        if self.server.is_some() {
            return Err(Error::AgentError(
                "Cannot change vaccel_config of initialized agent".to_string(),
            ));
        }
        let config_ref = self.vaccel_config.clone();
        let mut conf = config_ref.lock().unwrap();
        *conf = Some(config);
        Ok(self)
    }

    #[cfg(not(feature = "async"))]
    pub fn start(&mut self) -> Result<()> {
        if self.server.is_none() {
            self.vaccel_init()?;
            self.server_init()?;
        }

        self.server.as_mut().unwrap().start().map_err(|e| e.into())
    }

    #[cfg(feature = "async")]
    pub async fn start(&mut self) -> Result<()> {
        if self.server.is_none() {
            self.vaccel_init()?;
            self.server_init()?;
        }

        self.server
            .as_mut()
            .unwrap()
            .start()
            .await
            .map_err(|e| e.into())
    }

    #[cfg(not(feature = "async"))]
    pub fn stop(&mut self) -> Result<()> {
        match self.server.take() {
            Some(server) => {
                let server = server.stop_listen();
                self.server = Some(server);
                Ok(())
            }
            None => Err(Error::AgentError(
                "Cannot stop uninitialized agent".to_string(),
            )),
        }
    }

    #[cfg(feature = "async")]
    pub async fn stop(&mut self) -> Result<()> {
        match self.server.as_mut() {
            Some(server) => {
                server.stop_listen().await;
                Ok(())
            }
            None => Err(Error::AgentError(
                "Cannot stop uninitialized agent".to_string(),
            )),
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn shutdown(&mut self) -> Result<()> {
        match self.server.take() {
            Some(server) => {
                server.shutdown();
                Ok(())
            }
            None => Err(Error::AgentError(
                "Cannot shutdown uninitialized agent".to_string(),
            )),
        }
    }

    #[cfg(feature = "async")]
    pub async fn shutdown(&mut self) -> Result<()> {
        match self.server.take() {
            Some(mut server) => server.shutdown().await.map_err(|e| e.into()),
            None => Err(Error::AgentError(
                "Cannot shutdown uninitialized agent".to_string(),
            )),
        }
    }

    fn server_init(&mut self) -> Result<()> {
        if self.server.is_some() {
            return Err(Error::AgentError(
                "Server has already been initialized".to_string(),
            ));
        }

        if self.server_address.is_empty() {
            return Err(Error::AgentError(
                "Server address cannot be empty".to_string(),
            ));
        }

        let aservice = create_agent_service(Arc::new(AgentService::new()));
        let resolved_uri = resolve_uri(&self.server_address)?;

        let server: Server = Server::new()
            .bind(&resolved_uri)?
            .register_service(aservice);

        self.server = Some(server);

        Ok(())
    }

    fn vaccel_init(&mut self) -> Result<()> {
        let mut vaccel_config = self.vaccel_config.lock().unwrap();
        match vaccel_config.as_mut() {
            Some(c) => vaccel::bootstrap_with_config(c),
            None => {
                if !vaccel::is_initialized() {
                    vaccel::bootstrap()
                } else {
                    Ok(())
                }
            }
        }?;
        Ok(())
    }
}

fn resolve_uri(uri: &str) -> Result<String> {
    let parts: Vec<&str> = uri.split("://").collect();
    if parts.len() != 2 {
        return Err(Error::AgentError("Invalid server address uri".into()));
    }

    let scheme = parts[0].to_lowercase();
    match scheme.as_str() {
        "vsock" | "unix" => Ok(uri.to_string()),
        "tcp" => {
            let address = parts[1].to_lowercase();
            let mut resolved = match address.to_socket_addrs() {
                Ok(a) => a,
                Err(e) => return Err(Error::AgentError(e.to_string())),
            };
            let resolved_address = match resolved.next() {
                Some(a) => a.to_string(),
                None => {
                    return Err(Error::AgentError(
                        "Could not resolve TCP server address".into(),
                    ))
                }
            };

            Ok(format!("{}://{}", scheme, resolved_address.as_str()))
        }
        _ => Err(Error::AgentError("Unsupported protocol".into())),
    }
}
