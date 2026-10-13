use anyhow::{Context, Result};
use futures::channel::mpsc;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::protocol::WebSocket;
use tungstenite::{accept, Message};

/// Bokeh WebSocket server broadcasting FFT spectrum frames to browser clients
pub struct BokehServer {
    running: Arc<AtomicBool>,
}

impl BokehServer {
    pub fn spawn(listen_addr: String, mut rx: mpsc::UnboundedReceiver<Vec<f32>>) -> Result<Self> {
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

        // Thread 1: Accept incoming WebSocket client connections
        thread::spawn(move || {
            while running_acceptor.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        let _ = stream.set_nonblocking(false);
                        if let Ok(ws) = accept(stream) {
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

        // Thread 2: Broadcast incoming FFT frames to all connected WebSockets
        thread::spawn(move || {
            while running_c.load(Ordering::Relaxed) {
                match rx.try_recv() {
                    Ok(frame) => {
                        // Encode float slice directly into binary bytes (little-endian)
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
                        thread::sleep(std::time::Duration::from_millis(20));
                    }
                }
            }
        });

        Ok(Self { running })
    }
}

impl Drop for BokehServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
