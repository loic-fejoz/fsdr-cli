use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct PulseAudioConverter {}
impl BlockConverter for PulseAudioConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let dev_name = blk.parameter_or("device_name", "default");
        if blk.id.starts_with("pulse_source") {
            let blk = crate::blocks::PulseSource::new(dev_name);
            let blk = fg.add(blk)?.id();
            Ok(Box::new(DefaultPortAdapter::new(blk)))
        } else {
            let blk = crate::blocks::PulseSink::new(dev_name);
            let blk = fg.add(blk)?.id();
            Ok(Box::new(DefaultPortAdapter::new(blk)))
        }
    }
}

pub struct WeaverUsbFcConverter {}
impl BlockConverter for WeaverUsbFcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let audio_rate = Grc2FutureSdr::parameter_as_f32(blk, "audio_rate", "0.03125")?;
        let blk = crate::blocks::WeaverUsbModFc::new(audio_rate);
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

pub struct PulseShapingFilterCcConverter {}
impl BlockConverter for PulseShapingFilterCcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let filter_type = blk.parameter_or("type", "RRC");
        let alpha = Grc2FutureSdr::parameter_as_f32(blk, "alpha", "0.35")?;
        let sps = Grc2FutureSdr::parameter_as_f64(blk, "sps", "4")? as usize;
        let blk = crate::blocks::PulseShapingFilterCc::new(filter_type, alpha, sps, 8)?;
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

pub struct QpskModulatorConverter {}
impl BlockConverter for QpskModulatorConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let oqpsk = blk.id.contains("oqpsk");
        let sps = Grc2FutureSdr::parameter_as_f64(blk, "sps", "1")? as usize;
        let blk = crate::blocks::QpskModulatorU8C::new(oqpsk, sps);
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

pub struct QpskDemodConverter {}
impl BlockConverter for QpskDemodConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let oqpsk = blk.id.contains("oqpsk");
        let blk = crate::blocks::QpskDemodCU8::new(oqpsk);
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

pub struct QpskCostasLoopConverter {}
impl BlockConverter for QpskCostasLoopConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let oqpsk = blk.id.contains("oqpsk");
        let loop_bw = Grc2FutureSdr::parameter_as_f32(blk, "w", "0.02")?;
        let blk = crate::blocks::QpskCostasLoopCc::new(loop_bw, 0.707, oqpsk);
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

#[derive(Clone, Copy)]
pub struct G3ruhPortAdapter {
    blk: futuresdr::runtime::BlockId,
    is_framing: bool,
}

impl ConnectorAdapter for G3ruhPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(futuresdr::runtime::BlockId, &str)> {
        if self.is_framing {
            match port_name {
                "0" | "pdu" | "pdus" | "pdu_in" | "in" | "input" => Ok((self.blk, "pdu_in")),
                _ => anyhow::bail!("G3ruh Framing unknown input port: {port_name}"),
            }
        } else {
            match port_name {
                "0" | "in" | "input" => Ok((self.blk, "input")),
                _ => anyhow::bail!("G3ruh Deframing unknown input port: {port_name}"),
            }
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(futuresdr::runtime::BlockId, &str)> {
        if self.is_framing {
            match port_name {
                "0" | "out" | "output" => Ok((self.blk, "output")),
                _ => anyhow::bail!("G3ruh Framing unknown output port: {port_name}"),
            }
        } else {
            match port_name {
                "0" | "pdu" | "pdus" | "pdu_out" | "out" | "output" => Ok((self.blk, "pdu_out")),
                _ => anyhow::bail!("G3ruh Deframing unknown output port: {port_name}"),
            }
        }
    }
}

pub struct G3ruhConverter {}
impl BlockConverter for G3ruhConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        if blk.id == "pdu_framing_g3ruh" || blk.id.starts_with("pdu_framing") {
            let blk = crate::blocks::PduFramingG3ruh::new();
            let blk = fg.add(blk)?.id();
            Ok(Box::new(G3ruhPortAdapter {
                blk,
                is_framing: true,
            }))
        } else {
            let blk = crate::blocks::PduDeframingG3ruh::new();
            let blk = fg.add(blk)?.id();
            Ok(Box::new(G3ruhPortAdapter {
                blk,
                is_framing: false,
            }))
        }
    }
}

pub struct Afsk1200Converter {}
impl BlockConverter for Afsk1200Converter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let sample_rate = Grc2FutureSdr::parameter_as_f32(blk, "sample_rate", "48000")?;
        if blk.id == "afsk1200_mod_u8_f" || blk.id.starts_with("afsk1200_mod") {
            let blk = crate::blocks::Afsk1200ModU8F::new(sample_rate);
            let blk = fg.add(blk)?.id();
            Ok(Box::new(DefaultPortAdapter::new(blk)))
        } else {
            let blk = crate::blocks::Afsk1200DemodFU8::new(sample_rate);
            let blk = fg.add(blk)?.id();
            Ok(Box::new(DefaultPortAdapter::new(blk)))
        }
    }
}

#[derive(Clone, Copy)]
pub struct HdlcPortAdapter {
    blk: futuresdr::runtime::BlockId,
    is_framing: bool,
}

impl ConnectorAdapter for HdlcPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(futuresdr::runtime::BlockId, &str)> {
        if self.is_framing {
            match port_name {
                "0" | "pdu" | "pdus" | "pdu_in" | "in" | "input" => Ok((self.blk, "pdu_in")),
                _ => anyhow::bail!("HDLC Framing unknown input port: {port_name}"),
            }
        } else {
            match port_name {
                "0" | "in" | "input" => Ok((self.blk, "input")),
                _ => anyhow::bail!("HDLC Deframing unknown input port: {port_name}"),
            }
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(futuresdr::runtime::BlockId, &str)> {
        if self.is_framing {
            match port_name {
                "0" | "out" | "output" => Ok((self.blk, "output")),
                _ => anyhow::bail!("HDLC Framing unknown output port: {port_name}"),
            }
        } else {
            match port_name {
                "0" | "pdu" | "pdus" | "pdu_out" | "out" | "output" => Ok((self.blk, "pdu_out")),
                _ => anyhow::bail!("HDLC Deframing unknown output port: {port_name}"),
            }
        }
    }
}

pub struct HdlcConverter {}
impl BlockConverter for HdlcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        if blk.id == "hdlc_framing_pdu_u8" || blk.id.starts_with("hdlc_framing") {
            let blk = crate::blocks::HdlcFramingPduU8::new();
            let blk = fg.add(blk)?.id();
            Ok(Box::new(HdlcPortAdapter {
                blk,
                is_framing: true,
            }))
        } else {
            let blk = crate::blocks::HdlcDeframingU8Pdu::new();
            let blk = fg.add(blk)?.id();
            Ok(Box::new(HdlcPortAdapter {
                blk,
                is_framing: false,
            }))
        }
    }
}

pub struct FirInterpolateConverter {}
impl BlockConverter for FirInterpolateConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let interp = Grc2FutureSdr::parameter_as_f64(blk, "interp", "1")? as usize;
        let bw = Grc2FutureSdr::parameter_as_f32(blk, "transition_bw", "0.05")?;
        let window = blk.parameter_or("window", "HAMMING");
        let blk = crate::blocks::FirInterpolateCc::new(interp, bw, window);
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

pub struct BokehSpectrumSinkConverter {}
impl BlockConverter for BokehSpectrumSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let fft_size = Grc2FutureSdr::parameter_as_f64(blk, "fft_size", "4096")? as usize;
        let fps = Grc2FutureSdr::parameter_as_f64(blk, "fps", "20")? as u32;
        let blk = crate::blocks::BokehSpectrumSink::new(fft_size, fps);
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

pub struct SyntheticSdrSourceConverter {}
impl BlockConverter for SyntheticSdrSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let sample_rate = Grc2FutureSdr::parameter_as_f32(blk, "sample_rate", "2000000")?;
        let blk = crate::blocks::SyntheticSdrSource::new(sample_rate);
        let blk = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(blk)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_afsk1200_converter_mod_vs_demod() -> Result<()> {
        let mut fg = Flowgraph::new();
        let converter = Afsk1200Converter {};

        // 1. Modulator block
        let mod_instance = BlockInstance::new("afsk_mod", "afsk1200_mod_u8_f");
        let mod_adapter = converter.convert(&mod_instance, &mut fg)?;
        let (in_blk, in_port) = mod_adapter.adapt_input_port("0")?;
        let (_out_blk, out_port) = mod_adapter.adapt_output_port("0")?;
        assert_eq!(in_port, "input");
        assert_eq!(out_port, "output");

        // 2. Demodulator block (ensure it is NOT misclassified as modulator despite containing "mod")
        let demod_instance = BlockInstance::new("afsk_demod", "afsk1200_demod_f_u8");
        let demod_adapter = converter.convert(&demod_instance, &mut fg)?;
        let (d_in_blk, d_in_port) = demod_adapter.adapt_input_port("0")?;
        let (_d_out_blk, d_out_port) = demod_adapter.adapt_output_port("0")?;
        assert_eq!(d_in_port, "input");
        assert_eq!(d_out_port, "output");
        assert_ne!(in_blk, d_in_blk);

        Ok(())
    }

    #[test]
    fn test_hdlc_converter_framing_vs_deframing() -> Result<()> {
        let mut fg = Flowgraph::new();
        let converter = HdlcConverter {};

        let framing_instance = BlockInstance::new("hdlc_tx", "hdlc_framing_pdu_u8");
        let framing_adapter = converter.convert(&framing_instance, &mut fg)?;
        let (_, in_port) = framing_adapter.adapt_input_port("pdu_in")?;
        let (_, out_port) = framing_adapter.adapt_output_port("output")?;
        assert_eq!(in_port, "pdu_in");
        assert_eq!(out_port, "output");

        let deframing_instance = BlockInstance::new("hdlc_rx", "hdlc_deframing_u8_pdu");
        let deframing_adapter = converter.convert(&deframing_instance, &mut fg)?;
        let (_, d_in_port) = deframing_adapter.adapt_input_port("input")?;
        let (_, d_out_port) = deframing_adapter.adapt_output_port("pdu_out")?;
        assert_eq!(d_in_port, "input");
        assert_eq!(d_out_port, "pdu_out");

        Ok(())
    }
}
