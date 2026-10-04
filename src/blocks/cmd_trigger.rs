use super::power_tagger::tag_matches;
use anyhow::Result;
use fsdr_blocks::sigmf::{DatasetFormat, DescriptionBuilder};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Type trait to identify SigMF sample format
pub trait SigmfSample: CpuSample {
    fn dataset_format() -> DatasetFormat;
    fn as_bytes(slice: &[Self]) -> &[u8];
}

impl SigmfSample for f32 {
    fn dataset_format() -> DatasetFormat {
        DatasetFormat::Rf32Le
    }
    fn as_bytes(slice: &[Self]) -> &[u8] {
        let byte_len = std::mem::size_of_val(slice);
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, byte_len) }
    }
}

impl SigmfSample for Complex32 {
    fn dataset_format() -> DatasetFormat {
        DatasetFormat::Cf32Le
    }
    fn as_bytes(slice: &[Self]) -> &[u8] {
        let byte_len = std::mem::size_of_val(slice);
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, byte_len) }
    }
}

struct ActiveRecording {
    temp_dir: TempDir,
    data_file: File,
    meta_path: PathBuf,
    description: DescriptionBuilder,
    samples_recorded: usize,
}

impl ActiveRecording {
    fn start(
        datatype: DatasetFormat,
        samp_rate: f64,
        debug: bool,
        start_tag: &str,
    ) -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let data_path = dir.path().join("message.sigmf-data");
        let meta_path = dir.path().join("message.sigmf-meta");
        let data_file = File::create(&data_path)?;

        let mut desc = DescriptionBuilder::from(datatype);
        let _ = desc.sample_rate(samp_rate);

        if debug {
            eprintln!(
                "[cmd_trigger] Started recording on '{}' -> {}",
                start_tag,
                meta_path.display()
            );
        }

        Ok(Self {
            temp_dir: dir,
            data_file,
            meta_path,
            description: desc,
            samples_recorded: 0,
        })
    }

    fn write_samples<T: SigmfSample>(&mut self, samples: &[T]) -> Result<()> {
        let bytes = T::as_bytes(samples);
        self.data_file.write_all(bytes)?;
        self.samples_recorded += samples.len();
        Ok(())
    }

    fn stop_and_trigger(
        mut self,
        cmd_template: &str,
        debug: bool,
        end_tag: &str,
        tasks: &Arc<Mutex<Vec<std::thread::JoinHandle<()>>>>,
    ) -> Result<()> {
        self.data_file.flush()?;

        let desc_built = self.description.build()?;
        let mut meta_file = File::create(&self.meta_path)?;
        desc_built.to_writer_pretty(&mut meta_file)?;
        meta_file.flush()?;

        let resolved_path = self.meta_path.to_str().unwrap_or("").to_string();
        let cmd_str = cmd_template.replace("$input_file", &resolved_path);

        if debug {
            eprintln!(
                "[cmd_trigger] Stopped recording on '{}' ({} samples). Spawning: {}",
                end_tag, self.samples_recorded, cmd_str
            );
        }

        let temp_dir = self.temp_dir;
        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
        {
            Ok(mut child) => {
                let handle = std::thread::spawn(move || {
                    match child.wait() {
                        Ok(status) => {
                            if debug {
                                eprintln!("[cmd_trigger] Process exited with: {status}");
                            }
                        }
                        Err(err) => {
                            futuresdr::tracing::warn!("CmdTrigger: process wait error: {err}");
                        }
                    }
                    drop(temp_dir);
                });
                if let Ok(mut lock) = tasks.lock() {
                    lock.push(handle);
                }
            }
            Err(err) => {
                eprintln!("[cmd_trigger] Failed to spawn command '{}': {err}", cmd_str);
                futuresdr::tracing::warn!(
                    "CmdTrigger: failed to spawn command '{}': {err}",
                    cmd_str
                );
                drop(temp_dir);
            }
        }

        Ok(())
    }
}

/// CmdTrigger records a stream of samples into a SigMF dataset while triggered between
/// `start_tag` and `end_tag`, and launches an external command substituting `$input_file`.
#[derive(Block)]
pub struct CmdTrigger<T: SigmfSample> {
    #[input]
    input: DefaultCpuReader<T>,

    start_tag: String,
    end_tag: String,
    cmd_template: String,
    datatype: DatasetFormat,
    samp_rate: f64,
    recording: Option<ActiveRecording>,
    tasks: Arc<Mutex<Vec<std::thread::JoinHandle<()>>>>,
    debug: bool,
}

impl<T: SigmfSample> CmdTrigger<T> {
    pub fn new(start_tag: String, end_tag: String, cmd_template: String, samp_rate: f64) -> Self {
        Self::with_debug(start_tag, end_tag, cmd_template, samp_rate, false)
    }

    pub fn with_debug(
        start_tag: String,
        end_tag: String,
        cmd_template: String,
        samp_rate: f64,
        debug: bool,
    ) -> Self {
        let datatype = T::dataset_format();
        Self {
            input: DefaultCpuReader::default(),
            start_tag,
            end_tag,
            cmd_template,
            datatype,
            samp_rate,
            recording: None,
            tasks: Arc::new(Mutex::new(Vec::new())),
            debug,
        }
    }
}

#[doc(hidden)]
impl<T: SigmfSample> Kernel for CmdTrigger<T> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (i, in_tags) = self.input.slice_with_tags();
        let items = i.len();

        if items > 0 {
            if in_tags.is_empty() {
                if let Some(ref mut rec) = self.recording {
                    rec.write_samples(i)?;
                }
            } else {
                let mut start_indices = Vec::new();
                let mut end_indices = Vec::new();

                for tag in in_tags {
                    if tag.index < items {
                        if tag_matches(&tag.tag, &self.start_tag) {
                            start_indices.push(tag.index);
                        } else if tag_matches(&tag.tag, &self.end_tag) {
                            end_indices.push(tag.index);
                        }
                    }
                }

                let mut current_idx = 0;
                while current_idx < items {
                    if self.recording.is_none() {
                        if let Some(&start_pos) =
                            start_indices.iter().find(|&&pos| pos >= current_idx)
                        {
                            current_idx = start_pos;
                            self.recording = Some(ActiveRecording::start(
                                self.datatype,
                                self.samp_rate,
                                self.debug,
                                &self.start_tag,
                            )?);
                        } else {
                            break;
                        }
                    } else if let Some(&end_pos) =
                        end_indices.iter().find(|&&pos| pos >= current_idx)
                    {
                        if let Some(rec) = self.recording.take() {
                            let mut rec = rec;
                            rec.write_samples(&i[current_idx..=end_pos])?;
                            current_idx = end_pos + 1;
                            rec.stop_and_trigger(
                                &self.cmd_template,
                                self.debug,
                                &self.end_tag,
                                &self.tasks,
                            )?;
                        }
                    } else {
                        if let Some(ref mut rec) = self.recording {
                            rec.write_samples(&i[current_idx..items])?;
                        }
                        break;
                    }
                }
            }

            self.input.consume(items);
        }

        if self.input.finished() {
            if let Some(rec) = self.recording.take() {
                rec.stop_and_trigger(&self.cmd_template, self.debug, &self.end_tag, &self.tasks)?;
            }
            io.finished = true;
        }

        Ok(())
    }

    async fn deinit(&mut self, _mo: &mut MessageOutputs, _meta: &BlockMeta) -> Result<()> {
        if let Some(rec) = self.recording.take() {
            rec.stop_and_trigger(&self.cmd_template, self.debug, &self.end_tag, &self.tasks)?;
        }
        if let Ok(mut lock) = self.tasks.lock() {
            for handle in lock.drain(..) {
                let _ = handle.join();
            }
        }
        Ok(())
    }
}
