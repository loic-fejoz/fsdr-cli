use anyhow::{Context, Result};
use futures::channel::mpsc;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::protocol::WebSocket;
use tungstenite::{accept, Message};

pub type CommandHandler = Arc<dyn Fn(&str) -> Option<String> + Send + Sync + 'static>;

/// Bokeh WebSocket server broadcasting FFT spectrum frames and handling bidirectional events
pub struct BokehServer {
    running: Arc<AtomicBool>,
    event_tx: mpsc::UnboundedSender<String>,
}

impl BokehServer {
    pub fn spawn(listen_addr: String, rx: mpsc::UnboundedReceiver<Vec<f32>>) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded::<String>();
        Self::spawn_with_control(listen_addr, rx, event_tx, event_rx, None, None)
    }

    pub fn spawn_with_control(
        listen_addr: String,
        mut rx: mpsc::UnboundedReceiver<Vec<f32>>,
        event_tx: mpsc::UnboundedSender<String>,
        mut event_rx: mpsc::UnboundedReceiver<String>,
        initial_state_fn: Option<Arc<dyn Fn() -> String + Send + Sync + 'static>>,
        cmd_handler: Option<CommandHandler>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(&listen_addr)
            .with_context(|| format!("Failed to bind WebSocket server on {}", listen_addr))?;
        listener
            .set_nonblocking(true)
            .context("Failed to set non-blocking on WebSocket listener")?;

        let running = Arc::new(AtomicBool::new(true));
        let running_c = running.clone();

        let clients: Arc<Mutex<Vec<WebSocket<std::net::TcpStream>>>> =
            Arc::new(Mutex::new(Vec::new()));
        let clients_acceptor = clients.clone();
        let running_acceptor = running.clone();
        let init_state = initial_state_fn;

        // Thread 1: Accept incoming WebSocket client connections
        thread::spawn(move || {
            while running_acceptor.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        let _ = stream.set_nonblocking(false);
                        if let Ok(mut ws) = accept(stream) {
                            if let Some(ref get_state) = init_state {
                                let state_str = get_state();
                                let _ = ws.send(Message::Text(state_str.into()));
                            }
                            let _ = ws.get_ref().set_nonblocking(true);
                            if let Ok(mut list) = clients_acceptor.lock() {
                                list.push(ws);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
        });

        // Thread 2: Servicing loop (read client commands, broadcast events, broadcast FFT frames)
        thread::spawn(move || {
            while running_c.load(Ordering::Relaxed) {
                // 1. Process client incoming messages / commands (non-blocking)
                if let Ok(mut list) = clients.lock() {
                    let mut outgoing_replies = Vec::new();
                    list.retain_mut(|ws| {
                        loop {
                            match ws.read() {
                                Ok(Message::Text(text)) => {
                                    if let Some(ref handler) = cmd_handler {
                                        if let Some(reply) = handler(text.as_str()) {
                                            outgoing_replies.push(reply);
                                        }
                                    }
                                }
                                Ok(Message::Ping(data)) => {
                                    let _ = ws.send(Message::Pong(data));
                                }
                                Ok(Message::Close(_)) => return false,
                                Ok(_) => {}
                                Err(tungstenite::Error::Io(e))
                                    if e.kind() == std::io::ErrorKind::WouldBlock =>
                                {
                                    break;
                                }
                                Err(_) => return false,
                            }
                        }
                        true
                    });

                    // Send any replies
                    for reply in outgoing_replies {
                        let msg = Message::Text(reply.into());
                        list.retain_mut(|ws| ws.send(msg.clone()).is_ok());
                    }
                }

                // 2. Broadcast events from channel
                while let Ok(evt_str) = event_rx.try_recv() {
                    let msg = Message::Text(evt_str.into());
                    if let Ok(mut list) = clients.lock() {
                        list.retain_mut(|ws| ws.send(msg.clone()).is_ok());
                    }
                }

                // 3. Broadcast incoming FFT frames
                match rx.try_recv() {
                    Ok(frame) => {
                        let mut bytes = Vec::with_capacity(frame.len() * 4);
                        for sample in frame {
                            bytes.extend_from_slice(&sample.to_le_bytes());
                        }

                        let msg = Message::Binary(bytes.into());
                        if let Ok(mut list) = clients.lock() {
                            list.retain_mut(|ws| ws.send(msg.clone()).is_ok());
                        }
                    }
                    Err(e) if e.is_closed() => break,
                    Err(_) => {
                        thread::sleep(std::time::Duration::from_millis(15));
                    }
                }
            }
        });

        Ok(Self { running, event_tx })
    }

    pub fn broadcast(&self, msg: String) {
        let _ = self.event_tx.unbounded_send(msg);
    }

    pub fn event_sender(&self) -> mpsc::UnboundedSender<String> {
        self.event_tx.clone()
    }
}

impl Drop for BokehServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
