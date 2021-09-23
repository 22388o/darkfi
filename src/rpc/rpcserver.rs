use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::Arc;
use std::path::PathBuf;

use async_native_tls::{Identity, TlsAcceptor};
use async_trait::async_trait;
use log::{debug, error};
use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    Async,
};

use crate::rpc::jsonrpc::{JsonRequest, JsonResult};
use crate::Result;

pub struct RpcServerConfig {
    pub socket_addr: String,
    pub use_tls: bool,
    pub identity_path: PathBuf,
    pub identity_pass: String,
}

#[async_trait]
pub trait RequestHandler: Sync + Send {
    async fn handle_request(&self, req: JsonRequest) -> JsonResult;
}

async fn serve(
    mut stream: Async<TcpStream>,
    tls: Option<TlsAcceptor>,
    rh: Arc<impl RequestHandler + 'static>,
) -> Result<()> {
    debug!(target: "RPC SERVER", "Accepted connection");

    let mut buf = [0; 2048];

    match tls {
        None => loop {
            let n = match stream.read(&mut buf).await {
                Ok(n) if n == 0 => {
                    debug!(target: "RPC SERVER", "Closed connection");
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => {
                    debug!(target: "RPC SERVER", "Failed to read from socket: {:#?}", e);
                    debug!(target: "RPC SERVER", "Closed connection");
                    return Ok(());
                }
            };

            let r: JsonRequest = match serde_json::from_slice(&buf[0..n]) {
                Ok(r) => r,
                Err(e) => {
                    debug!(target: "RPC SERVER", "Received invalid JSON: {:#?}", e);
                    debug!(target: "RPC SERVER", "Closed connection");
                    return Ok(());
                }
            };

            let reply = rh.handle_request(r).await;
            let j = serde_json::to_string(&reply).unwrap();
            debug!(target: "RPC", "<-- {}", j);

            if let Err(e) = stream.write_all(j.as_bytes()).await {
                debug!(target: "RPC SERVER", "Failed to write to socket: {:#?}", e);
                debug!(target: "RPC SERVER", "Closed connection");
                return Ok(());
            }
        },
        Some(tls) => match tls.accept(stream).await {
            Ok(mut stream) => loop {
                let n = match stream.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        debug!(target: "RPC SERVER", "Closed connection");
                        return Ok(());
                    }
                    Ok(n) => n,
                    Err(e) => {
                        debug!(target: "RPC SERVER", "Failed to read from socket: {:#?}", e);
                        debug!(target: "RPC SERVER", "Closed connection");
                        return Ok(());
                    }
                };

                let r: JsonRequest = match serde_json::from_slice(&buf[0..n]) {
                    Ok(r) => r,
                    Err(e) => {
                        debug!(target: "RPC SERVER", "Received invalid JSON: {:#?}", e);
                        debug!(target: "RPC SERVER", "Closed connection");
                        return Ok(());
                    }
                };

                let reply = rh.handle_request(r).await;
                let j = serde_json::to_string(&reply).unwrap();
                debug!(target: "RPC", "<-- {:?}", j);

                if let Err(e) = stream.write_all(j.as_bytes()).await {
                    debug!(target: "RPC SERVER", "Failed to write to socket: {:#?}", e);
                    return Ok(());
                }
            },
            Err(e) => {
                debug!(target: "RPC SERVER", "Failed to establish TLS connection: {:#}", e);
                return Ok(());
            }
        },
    }
}

async fn listen(
    listener: Async<TcpListener>,
    tls: Option<TlsAcceptor>,
    rh: Arc<impl RequestHandler + 'static>,
) -> Result<()> {
    match &tls {
        None => {
            debug!(target: "RPC SERVER", "Listening on tcp://{}", listener.get_ref().local_addr()?)
        }
        Some(_) => {
            debug!(target: "RPC SERVER", "Listening on tls://{}", listener.get_ref().local_addr()?)
        }
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let tls = tls.clone();
        let rh_c = rh.clone();

        smol::spawn(async move {
            if let Err(err) = serve(stream, tls, rh_c).await {
                error!(target: "RPC SERVER", "Connection error: {:#?}", err);
            }
        })
        .detach();
    }
}

pub async fn listen_and_serve(
    cfg: RpcServerConfig,
    rh: impl RequestHandler + 'static,
) -> Result<()> {
    let tls: Option<TlsAcceptor>;

    let sockaddr = SocketAddr::from_str(&cfg.socket_addr)?;

    if cfg.use_tls {
        let ident_bytes = std::fs::read(cfg.identity_path)?;
        let identity = Identity::from_pkcs12(&ident_bytes, &cfg.identity_pass)?;
        tls = Some(TlsAcceptor::from(native_tls::TlsAcceptor::new(identity)?));
    } else {
        tls = None;
    }

    let rh = Arc::new(rh);
    let listener = listen(Async::<TcpListener>::bind(sockaddr)?, tls, rh);
    listener.await
}
