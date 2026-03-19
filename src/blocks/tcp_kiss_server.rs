use anyhow::Result;
use futuresdr::prelude::*;
use futuresdr::runtime::Pmt;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

enum Event {
    Frame(Vec<u8>),
    NewClient(TcpStream),
}

#[derive(Block)]
#[message_inputs(in_port)]
pub struct TcpKissServer {
    tx: mpsc::Sender<Event>,
}

impl TcpKissServer {
    pub fn new(address: &str) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<Event>();
        let addr = address.to_string();
        let tx_acceptor = tx.clone();

        // Acceptor thread
        thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind(&addr) {
                for stream in listener.incoming().flatten() {
                    let _ = tx_acceptor.send(Event::NewClient(stream));
                }
            }
        });

        // Distributor thread
        thread::spawn(move || {
            let mut clients = Vec::new();

            while let Ok(event) = rx.recv() {
                match event {
                    Event::NewClient(stream) => {
                        clients.push(stream);
                    }
                    Event::Frame(data) => {
                        let mut escaped = Vec::with_capacity(data.len() + 2);
                        escaped.push(0xC0); // FEND
                        for &byte in &data {
                            if byte == 0xC0 {
                                escaped.push(0xDB);
                                escaped.push(0xDC);
                            } else if byte == 0xDB {
                                escaped.push(0xDB);
                                escaped.push(0xDD);
                            } else {
                                escaped.push(byte);
                            }
                        }
                        escaped.push(0xC0); // FEND

                        clients.retain_mut(|stream| {
                            stream.write_all(&escaped).is_ok()
                        });
                    }
                }
            }
        });

        Ok(Self { tx })
    }

    async fn in_port(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        match p {
            Pmt::Blob(data) => {
                let _ = self.tx.send(Event::Frame(data));
            }
            Pmt::Finished => {
                io.finished = true;
            }
            _ => {}
        }
        Ok(Pmt::Null)
    }
}

#[doc(hidden)]
impl Kernel for TcpKissServer {}
