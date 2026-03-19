use anyhow::Result;
use futures::channel::mpsc;
use futures::StreamExt;
use futuresdr::prelude::*;
use futuresdr::runtime::Pmt;
use std::io::Read;
use std::net::TcpStream;
use std::thread;

#[derive(Block)]
#[message_outputs(out)]
pub struct TcpKissClient {
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
}

impl TcpKissClient {
    pub fn new(address: &str) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded::<Vec<u8>>();
        let addr = address.to_string();

        thread::spawn(move || {
            if let Ok(mut stream) = TcpStream::connect(&addr) {
                let mut buffer = [0u8; 4096];
                let mut current_frame = Vec::new();
                let mut escape = false;

                while let Ok(n) = stream.read(&mut buffer) {
                    if n == 0 {
                        break; // EOF
                    }
                    for &byte in &buffer[..n] {
                        if byte == 0xC0 {
                            if !current_frame.is_empty() {
                                let _ = tx.unbounded_send(current_frame.clone());
                                current_frame.clear();
                            }
                        } else if byte == 0xDB {
                            escape = true;
                        } else {
                            if escape {
                                if byte == 0xDC {
                                    current_frame.push(0xC0);
                                } else if byte == 0xDD {
                                    current_frame.push(0xDB);
                                } else {
                                    current_frame.push(0xDB);
                                    current_frame.push(byte);
                                }
                                escape = false;
                            } else {
                                current_frame.push(byte);
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { rx })
    }
}

#[doc(hidden)]
impl Kernel for TcpKissClient {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        match self.rx.next().await {
            Some(frame) => {
                mio.post("out", Pmt::Blob(frame)).await?;
                io.call_again = true;
            }
            None => {
                io.finished = true;
            }
        }
        Ok(())
    }
}
