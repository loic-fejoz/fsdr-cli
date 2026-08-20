use anyhow::Result;
use futures::channel::mpsc;
use futures::StreamExt;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::Pmt;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Block)]
#[message_outputs(variables)]
pub struct CatServer {
    rx: mpsc::UnboundedReceiver<String>,
}

impl CatServer {
    pub fn new(
        port: u16,
        initial_rx_freq: u64,
        initial_tx_freq: u64,
        initial_ctcss: i32,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded::<String>();
        let state = Arc::new(Mutex::new(CatState {
            rx_freq: initial_rx_freq,
            tx_freq: initial_tx_freq,
            ctcss_tone: initial_ctcss, // tenths of Hz
            ctcss_sql: initial_ctcss,
            dcs_code: 23,
            dcs_sql: 23,
        }));

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;

        // Broadcast initial state
        let _ = tx.unbounded_send(format!("rx_freq={}", initial_rx_freq));
        let _ = tx.unbounded_send(format!("tx_freq={}", initial_tx_freq));
        let _ = tx.unbounded_send(format!("ctcss_tone={}", initial_ctcss as f32 / 10.0));
        let _ = tx.unbounded_send(format!("ctcss_sql={}", initial_ctcss as f32 / 10.0));

        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let tx = tx.clone();
                let state = state.clone();

                thread::spawn(move || {
                    let reader_stream = match stream.try_clone() {
                        Ok(s) => s,
                        Err(_) => return,
                    };
                    let mut reader = BufReader::new(reader_stream);
                    let mut writer = stream;
                    let mut line = String::new();

                    while let Ok(bytes_read) = reader.read_line(&mut line) {
                        if bytes_read == 0 {
                            break;
                        }
                        let cmd = line.trim();

                        if let Some(stripped) = cmd.strip_prefix("F ") {
                            if let Ok(freq) = stripped.trim().parse::<u64>() {
                                state.lock().unwrap().rx_freq = freq;
                                let _ = tx.unbounded_send(format!("rx_freq={}", freq));
                                let _ = writer.write_all(b"RPRT 0\n");
                            } else {
                                let _ = writer.write_all(b"RPRT 1\n");
                            }
                        } else if let Some(stripped) = cmd.strip_prefix("I ") {
                            if let Ok(freq) = stripped.trim().parse::<u64>() {
                                state.lock().unwrap().tx_freq = freq;
                                let _ = tx.unbounded_send(format!("tx_freq={}", freq));
                                let _ = writer.write_all(b"RPRT 0\n");
                            } else {
                                let _ = writer.write_all(b"RPRT 1\n");
                            }
                        } else if let Some(stripped) = cmd.strip_prefix("C ") {
                            if let Ok(tone) = stripped.trim().parse::<i32>() {
                                state.lock().unwrap().ctcss_tone = tone;
                                let _ =
                                    tx.unbounded_send(format!("ctcss_tone={}", tone as f32 / 10.0));
                                let _ = writer.write_all(b"RPRT 0\n");
                            } else {
                                let _ = writer.write_all(b"RPRT 1\n");
                            }
                        } else if let Some(stripped) = cmd.strip_prefix("D ") {
                            if let Ok(code) = stripped.trim().parse::<i32>() {
                                state.lock().unwrap().dcs_code = code;
                                let _ = tx.unbounded_send(format!("dcs_code={}", code));
                                let _ = writer.write_all(b"RPRT 0\n");
                            } else {
                                let _ = writer.write_all(b"RPRT 1\n");
                            }
                        } else if let Some(stripped) = cmd.strip_prefix("\\set_ctcss_sql ") {
                            if let Ok(tone) = stripped.trim().parse::<i32>() {
                                state.lock().unwrap().ctcss_sql = tone;
                                let _ =
                                    tx.unbounded_send(format!("ctcss_sql={}", tone as f32 / 10.0));
                                let _ = writer.write_all(b"RPRT 0\n");
                            } else {
                                let _ = writer.write_all(b"RPRT 1\n");
                            }
                        } else if let Some(stripped) = cmd.strip_prefix("\\set_dcs_sql ") {
                            if let Ok(code) = stripped.trim().parse::<i32>() {
                                state.lock().unwrap().dcs_sql = code;
                                let _ = tx.unbounded_send(format!("dcs_sql={}", code));
                                let _ = writer.write_all(b"RPRT 0\n");
                            } else {
                                let _ = writer.write_all(b"RPRT 1\n");
                            }
                        } else if cmd == "f" {
                            let freq = state.lock().unwrap().rx_freq;
                            let _ = writer.write_all(format!("{}\n", freq).as_bytes());
                        } else if cmd == "i" {
                            let freq = state.lock().unwrap().tx_freq;
                            let _ = writer.write_all(format!("{}\n", freq).as_bytes());
                        } else if cmd == "c" {
                            let tone = state.lock().unwrap().ctcss_tone;
                            let _ = writer.write_all(format!("{}\n", tone).as_bytes());
                        } else if cmd == "d" {
                            let code = state.lock().unwrap().dcs_code;
                            let _ = writer.write_all(format!("{}\n", code).as_bytes());
                        } else if cmd == "\\get_ctcss_sql" {
                            let tone = state.lock().unwrap().ctcss_sql;
                            let _ = writer.write_all(format!("{}\n", tone).as_bytes());
                        } else if cmd == "\\get_dcs_sql" {
                            let code = state.lock().unwrap().dcs_sql;
                            let _ = writer.write_all(format!("{}\n", code).as_bytes());
                        } else if cmd == "v" {
                            let _ = writer.write_all(b"VFOA\n");
                        } else if cmd == "m" {
                            let _ = writer.write_all(b"FM 15000\n");
                        } else {
                            // Ignore unknown commands, return error or empty to keep client happy
                            let _ = writer.write_all(b"RPRT 1\n");
                        }
                        line.clear();
                    }
                });
            }
        });

        Ok(Self { rx })
    }
}

struct CatState {
    rx_freq: u64,
    tx_freq: u64,
    ctcss_tone: i32,
    ctcss_sql: i32,
    dcs_code: i32,
    dcs_sql: i32,
}

#[doc(hidden)]
impl Kernel for CatServer {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        match self.rx.next().await {
            Some(msg) => {
                mio.post("variables", Pmt::String(msg)).await?;
                io.call_again = true;
            }
            None => {
                io.finished = true;
            }
        }
        Ok(())
    }
}
