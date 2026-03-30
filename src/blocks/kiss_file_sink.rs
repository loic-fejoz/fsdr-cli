use anyhow::Result;
use futuresdr::prelude::*;
use futuresdr::runtime::Pmt;
use std::fs::File;
use std::io::Write;

#[derive(Block)]
#[message_inputs(in_port)]
pub struct KissFileSink {
    file: Option<File>,
}

impl KissFileSink {
    pub fn new(filename: &str) -> Result<Self> {
        let file = if filename == "-" || filename == "/proc/self/fd/1" || filename.is_empty() {
            None
        } else {
            Some(File::create(filename)?)
        };
        Ok(Self { file })
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
                let mut escaped = Vec::with_capacity(data.len() * 2 + 3);
                escaped.push(0xC0); // FEND
                escaped.push(0x00); // Command byte (Port 0, Data)
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

                if let Some(ref mut f) = self.file {
                    f.write_all(&escaped)?;
                } else {
                    let mut stdout = std::io::stdout();
                    stdout.write_all(&escaped)?;
                }
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
impl Kernel for KissFileSink {}
