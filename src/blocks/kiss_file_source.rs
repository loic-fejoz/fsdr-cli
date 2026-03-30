use anyhow::Result;
use futuresdr::prelude::*;
use futuresdr::runtime::Pmt;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

#[derive(Block)]
#[message_outputs(output)]
pub struct KissFileSource {
    frames: VecDeque<Vec<u8>>,
}

impl KissFileSource {
    pub fn new(filename: &str) -> Result<Self> {
        let mut frames = VecDeque::new();

        let mut buffer = Vec::new();
        if filename == "/proc/self/fd/0" || filename == "-" || filename.is_empty() {
            let mut stdin = std::io::stdin();
            let _ = stdin.read_to_end(&mut buffer)?;
        } else {
            let mut file = File::open(filename)?;
            let _ = file.read_to_end(&mut buffer)?;
        }

        let mut current_frame = Vec::new();
        let mut escape = false;

        for &byte in &buffer {
            if byte == 0xC0 {
                if !current_frame.is_empty() {
                    // Strip the command byte (the first byte)
                    let data = current_frame[1..].to_vec();
                    frames.push_back(data);
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
                        // Invalid escape sequence, but keep both bytes
                        current_frame.push(0xDB);
                        current_frame.push(byte);
                    }
                    escape = false;
                } else {
                    current_frame.push(byte);
                }
            }
        }

        Ok(Self { frames })
    }
}

#[doc(hidden)]
impl Kernel for KissFileSource {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        if let Some(frame) = self.frames.pop_front() {
            mio.post("output", Pmt::Blob(frame)).await?;
            io.call_again = true;
        } else {
            io.finished = true;
        }
        Ok(())
    }
}
