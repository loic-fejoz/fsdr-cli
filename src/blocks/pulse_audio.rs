use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, Stream, StreamConfig};
use futures::channel::mpsc;
use futuresdr::runtime::dev::prelude::*;

/// PulseSource: Captures audio from PulseAudio / PipeWire device by name into f32 stream
#[derive(Block)]
pub struct PulseSource<O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>> {
    _stream: Option<Stream>,
    rx: mpsc::UnboundedReceiver<f32>,
    #[output]
    output: O,
}

impl PulseSource<DefaultCpuWriter<f32>> {
    pub fn new(device_name: &str) -> Self {
        let (tx, rx) = mpsc::unbounded::<f32>();
        let host = cpal::default_host();
        let mut chosen_device = None;

        if let Ok(devices) = host.input_devices() {
            for dev in devices {
                let name = dev
                    .description()
                    .map(|d| d.name().to_string())
                    .unwrap_or_else(|_| dev.to_string());
                if name.eq_ignore_ascii_case(device_name) || name.contains(device_name) {
                    chosen_device = Some(dev);
                    break;
                }
            }
        }

        let stream = if let Some(dev) = chosen_device {
            let config = StreamConfig {
                channels: 1,
                sample_rate: 48000,
                buffer_size: BufferSize::Default,
            };
            let stream = dev
                .build_input_stream(
                    config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        for &s in data {
                            let _ = tx.unbounded_send(s);
                        }
                    },
                    |_err| {},
                    None,
                )
                .ok();
            if let Some(ref s) = stream {
                use cpal::traits::StreamTrait;
                let _ = s.play();
            }
            stream
        } else {
            // Fallback: spawn pacat recording process from named PulseAudio source
            let dev_name = device_name.to_string();
            std::thread::spawn(move || {
                use std::io::Read;
                let mut cmd = std::process::Command::new("pacat");
                cmd.arg("-r")
                    .arg("--format=float32le")
                    .arg("--rate=48000")
                    .arg("--channels=1");
                if dev_name != "default" && !dev_name.is_empty() {
                    cmd.arg("-d").arg(&dev_name);
                }
                cmd.stdout(std::process::Stdio::piped())
                    .stdin(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());

                if let Ok(mut child) = cmd.spawn() {
                    if let Some(mut stdout) = child.stdout.take() {
                        let mut byte_buf = [0u8; 1024 * 4];
                        while let Ok(n) = stdout.read(&mut byte_buf) {
                            if n == 0 {
                                break;
                            }
                            for chunk in byte_buf[..n].as_chunks::<4>().0 {
                                let s = f32::from_le_bytes(*chunk);
                                if tx.unbounded_send(s).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    let _ = child.kill();
                }
            });
            None
        };

        Self {
            _stream: stream,
            rx,
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<O> Kernel for PulseSource<O>
where
    O: CpuBufferWriter<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let out_slice = self.output.slice();
        let mut count = 0;

        while count < out_slice.len() {
            match self.rx.try_recv() {
                Ok(sample) => {
                    out_slice[count] = sample;
                    count += 1;
                }
                _ => break,
            }
        }

        if count > 0 {
            self.output.produce(count);
            io.call_again = true;
        }

        Ok(())
    }
}

/// PulseSink: Plays audio from f32 stream into PulseAudio / PipeWire device by name
#[derive(Block)]
pub struct PulseSink<I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>> {
    _stream: Option<Stream>,
    tx: mpsc::UnboundedSender<f32>,
    #[input]
    input: I,
}

impl PulseSink<DefaultCpuReader<f32>> {
    pub fn new(device_name: &str) -> Self {
        let (tx, mut rx) = mpsc::unbounded::<f32>();
        let host = cpal::default_host();
        let mut chosen_device = None;

        if let Ok(devices) = host.output_devices() {
            for dev in devices {
                let name = dev
                    .description()
                    .map(|d| d.name().to_string())
                    .unwrap_or_else(|_| dev.to_string());
                if name.eq_ignore_ascii_case(device_name) || name.contains(device_name) {
                    chosen_device = Some(dev);
                    break;
                }
            }
        }

        let stream = if let Some(dev) = chosen_device {
            let config = StreamConfig {
                channels: 1,
                sample_rate: 48000,
                buffer_size: BufferSize::Default,
            };
            let stream = dev
                .build_output_stream(
                    config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        for s in data.iter_mut() {
                            *s = rx.try_recv().ok().unwrap_or(0.0);
                        }
                    },
                    |_err| {},
                    None,
                )
                .ok();
            if let Some(ref s) = stream {
                use cpal::traits::StreamTrait;
                let _ = s.play();
            }
            stream
        } else {
            // Fallback: spawn pacat playback process to named PulseAudio sink
            let dev_name = device_name.to_string();
            std::thread::spawn(move || {
                use futures::StreamExt;
                use std::io::Write;
                let mut cmd = std::process::Command::new("pacat");
                cmd.arg("-p")
                    .arg("--format=float32le")
                    .arg("--rate=48000")
                    .arg("--channels=1");
                if dev_name != "default" && !dev_name.is_empty() {
                    cmd.arg("-d").arg(&dev_name);
                }
                cmd.stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());

                if let Ok(mut child) = cmd.spawn() {
                    if let Some(mut stdin) = child.stdin.take() {
                        let mut byte_buf = Vec::with_capacity(1024 * 4);
                        while let Some(sample) = futures::executor::block_on(rx.next()) {
                            byte_buf.extend_from_slice(&sample.to_le_bytes());
                            while let Ok(s) = rx.try_recv() {
                                byte_buf.extend_from_slice(&s.to_le_bytes());
                                if byte_buf.len() >= 4096 {
                                    break;
                                }
                            }
                            if stdin.write_all(&byte_buf).is_err() {
                                break;
                            }
                            let _ = stdin.flush();
                            byte_buf.clear();
                        }
                    }
                    let _ = child.kill();
                }
            });
            None
        };

        Self {
            _stream: stream,
            tx,
            input: DefaultCpuReader::default(),
        }
    }
}

#[doc(hidden)]
impl<I> Kernel for PulseSink<I>
where
    I: CpuBufferReader<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let m = in_slice.len();
        if m > 0 {
            for &sample in in_slice {
                let _ = self.tx.unbounded_send(sample);
            }
            self.input.consume(m);
        }

        if !self.input.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_cpal_devices() {
        for host_id in cpal::available_hosts() {
            println!("Available host: {:?}", host_id);
            let host = cpal::host_from_id(host_id).unwrap();
            if let Ok(devices) = host.output_devices() {
                for dev in devices {
                    let name = dev
                        .description()
                        .map(|d| d.name().to_string())
                        .unwrap_or_else(|_| dev.to_string());
                    if name.contains("Pulse") || name.contains("HF_") {
                        println!("  Device [{:?}]: {}", host_id, name);
                    }
                }
            }
        }
    }
}
