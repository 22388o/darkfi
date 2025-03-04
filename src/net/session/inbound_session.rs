use async_std::sync::Mutex;
use async_trait::async_trait;
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Weak},
};

use async_executor::Executor;
use log::{error, info};

use crate::{
    error::{Error, Result},
    net::{
        session::{Session, SessionBitflag, SESSION_INBOUND},
        Acceptor, AcceptorPtr, ChannelPtr, P2p,
    },
    system::{StoppableTask, StoppableTaskPtr},
};

struct InboundInfo {
    channel: ChannelPtr,
}

impl InboundInfo {
    async fn get_info(&self) -> serde_json::Value {
        self.channel.get_info().await
    }
}

/// Defines inbound connections session.
pub struct InboundSession {
    p2p: Weak<P2p>,
    acceptor: AcceptorPtr,
    accept_task: StoppableTaskPtr,
    connect_infos: Mutex<HashMap<SocketAddr, InboundInfo>>,
}

impl InboundSession {
    /// Create a new inbound session.
    pub fn new(p2p: Weak<P2p>) -> Arc<Self> {
        let acceptor = Acceptor::new();

        Arc::new(Self {
            p2p,
            acceptor,
            accept_task: StoppableTask::new(),
            connect_infos: Mutex::new(HashMap::new()),
        })
    }

    /// Starts the inbound session. Begins by accepting connections and fails if
    /// the address is not configured. Then runs the channel subscription
    /// loop.
    pub fn start(self: Arc<Self>, executor: Arc<Executor<'_>>) -> Result<()> {
        match self.p2p().settings().inbound {
            Some(accept_addr) => {
                self.clone().start_accept_session(accept_addr, executor.clone())?;
            }
            None => {
                info!(target: "net", "Not configured for accepting incoming connections.");
                return Ok(())
            }
        }

        self.accept_task.clone().start(
            self.clone().channel_sub_loop(executor.clone()),
            // Ignore stop handler
            |_| async {},
            Error::ServiceStopped,
            executor,
        );

        Ok(())
    }
    /// Stops the inbound session.
    pub async fn stop(&self) {
        self.acceptor.stop().await;
        self.accept_task.stop().await;
    }
    /// Start accepting connections for inbound session.
    fn start_accept_session(
        self: Arc<Self>,
        accept_addr: SocketAddr,
        executor: Arc<Executor<'_>>,
    ) -> Result<()> {
        info!(target: "net", "Starting inbound session on {}", accept_addr);
        let result = self.acceptor.clone().start(accept_addr, executor);
        if let Err(err) = result.clone() {
            error!(target: "net", "Error starting listener: {}", err);
        }
        result
    }

    /// Wait for all new channels created by the acceptor and call
    /// setup_channel() on them.
    async fn channel_sub_loop(self: Arc<Self>, executor: Arc<Executor<'_>>) -> Result<()> {
        let channel_sub = self.acceptor.clone().subscribe().await;
        loop {
            let channel = channel_sub.receive().await?;
            // Spawn a detached task to process the channel
            // This will just perform the channel setup then exit.
            executor.spawn(self.clone().setup_channel(channel, executor.clone())).detach();
        }
    }

    /// Registers the channel. First performs a network handshake and starts the
    /// channel. Then starts sending keep-alive and address messages across the
    /// channel.
    async fn setup_channel(
        self: Arc<Self>,
        channel: ChannelPtr,
        executor: Arc<Executor<'_>>,
    ) -> Result<()> {
        info!(target: "net", "Connected inbound [{}]", channel.address());

        self.clone().register_channel(channel.clone(), executor.clone()).await?;

        self.manage_channel_for_get_info(channel).await;

        Ok(())
    }

    async fn manage_channel_for_get_info(&self, channel: ChannelPtr) {
        let key = channel.address();
        self.connect_infos.lock().await.insert(key, InboundInfo { channel: channel.clone() });

        let stop_sub = channel.subscribe_stop().await;
        stop_sub.receive().await;

        self.connect_infos.lock().await.remove(&key);
    }
}

#[async_trait]
impl Session for InboundSession {
    async fn get_info(&self) -> serde_json::Value {
        let mut infos = HashMap::new();
        for (addr, info) in self.connect_infos.lock().await.iter() {
            infos.insert(addr.to_string(), info.get_info().await);
        }

        json!({
            "connected": infos,
        })
    }

    fn p2p(&self) -> Arc<P2p> {
        self.p2p.upgrade().unwrap()
    }

    fn selector_id(&self) -> SessionBitflag {
        SESSION_INBOUND
    }
}
