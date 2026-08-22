#![allow(clippy::type_complexity)]
use futuresdr::futuredsp::firdes;
use futuresdr::futuredsp::prelude::*;
use futuresdr::futuredsp::ComputationStatus;
use futuresdr::futuredsp::DecimatingFirFilter;
use futuresdr::futuredsp::FirFilter;
use futuresdr::futuredsp::PolyphaseResamplingFir;

use futuresdr::runtime::dev::prelude::*;

/// FIR filter with Stream Tag propagation support.
#[derive(Block)]
pub struct TagFir<
    InputType,
    OutputType,
    TapType,
    Core,
    IN = DefaultCpuReader<InputType>,
    OUT = DefaultCpuWriter<OutputType>,
> where
    InputType: CpuSample,
    OutputType: CpuSample,
    TapType: 'static + Send,
    Core: Filter<InputType, OutputType, TapType> + Send,
    IN: CpuBufferReader<Item = InputType>,
    OUT: CpuBufferWriter<Item = OutputType>,
{
    #[input]
    input: IN,
    #[output]
    output: OUT,
    filter: Core,
    interp: usize,
    decim: usize,
    _tap_type: std::marker::PhantomData<TapType>,
}

impl<InputType, OutputType, TapType, Core, IN, OUT>
    TagFir<InputType, OutputType, TapType, Core, IN, OUT>
where
    InputType: CpuSample,
    OutputType: CpuSample,
    TapType: 'static + Send,
    Core: Filter<InputType, OutputType, TapType> + Send + 'static,
    IN: CpuBufferReader<Item = InputType>,
    OUT: CpuBufferWriter<Item = OutputType>,
{
    /// Create FIR block
    pub fn new(filter: Core, interp: usize, decim: usize) -> Self {
        let mut input = IN::default();
        input.set_min_items(filter.length());
        Self {
            input,
            output: OUT::default(),
            filter,
            interp,
            decim,
            _tap_type: std::marker::PhantomData,
        }
    }

    /// Returns the number of taps
    pub fn n_taps(&self) -> usize {
        self.filter.length()
    }
}

#[doc(hidden)]
impl<InputType, OutputType, TapType, Core, IN, OUT> Kernel
    for TagFir<InputType, OutputType, TapType, Core, IN, OUT>
where
    InputType: CpuSample,
    OutputType: CpuSample,
    TapType: 'static + Send,
    Core: Filter<InputType, OutputType, TapType> + Send + 'static,
    IN: CpuBufferReader<Item = InputType>,
    OUT: CpuBufferWriter<Item = OutputType>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (i, in_tags) = self.input.slice_with_tags();
        let (o, mut out_tags) = self.output.slice_with_tags();

        let (consumed, produced, status) = self.filter.filter(i, o);

        if consumed > 0 && produced > 0 {
            for tag in in_tags {
                if tag.index < consumed {
                    let out_idx =
                        std::cmp::min((tag.index * self.interp) / self.decim, produced - 1);
                    out_tags.add_tag(out_idx, tag.tag.clone());
                }
            }
        }

        self.input.consume(consumed);
        self.output.produce(produced);

        if self.input.finished() && !matches!(status, ComputationStatus::InsufficientOutput) {
            io.finished = true;
        }

        Ok(())
    }
}

/// Builder for [`TagFir`] filter.
pub struct TagFirBuilder;

impl TagFirBuilder {
    /// Create a new non-resampling FIR filter with the specified taps.
    pub fn fir<InputType, OutputType, TapsType>(
        taps: TapsType,
    ) -> TagFir<InputType, OutputType, TapsType::TapType, FirFilter<InputType, OutputType, TapsType>>
    where
        InputType: CpuSample,
        OutputType: CpuSample,
        TapsType: 'static + Taps + Send,
        TapsType::TapType: 'static + Send,
        FirFilter<InputType, OutputType, TapsType>:
            Filter<InputType, OutputType, TapsType::TapType>,
    {
        TagFir::<InputType, OutputType, TapsType::TapType, FirFilter<InputType, OutputType, TapsType>>::new(
            FirFilter::new(taps),
            1,
            1,
        )
    }

    /// Create a decimating FIR filter with standard low-pass taps.
    pub fn decimating<InputType, OutputType, TapsType>(
        decim: usize,
    ) -> TagFir<InputType, OutputType, f32, DecimatingFirFilter<InputType, OutputType, Vec<f32>>>
    where
        InputType: CpuSample,
        OutputType: CpuSample,
        DecimatingFirFilter<InputType, OutputType, Vec<f32>>: Filter<InputType, OutputType, f32>,
    {
        let taps = firdes::kaiser::lowpass::<f32>(1.0 / decim as f64, 0.1, 0.0001);
        TagFirBuilder::decimating_with_taps(decim, taps)
    }

    /// Create a decimating FIR filter with the specified taps.
    pub fn decimating_with_taps<InputType, OutputType, TapsType>(
        decim: usize,
        taps: TapsType,
    ) -> TagFir<
        InputType,
        OutputType,
        TapsType::TapType,
        DecimatingFirFilter<InputType, OutputType, TapsType>,
    >
    where
        InputType: CpuSample,
        OutputType: CpuSample,
        TapsType: 'static + Taps + Send,
        TapsType::TapType: 'static + Send,
        DecimatingFirFilter<InputType, OutputType, TapsType>:
            Filter<InputType, OutputType, TapsType::TapType>,
    {
        TagFir::<
            InputType,
            OutputType,
            TapsType::TapType,
            DecimatingFirFilter<InputType, OutputType, TapsType>,
        >::new(DecimatingFirFilter::new(decim, taps), 1, decim)
    }
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

impl TagFirBuilder {
    /// rate by a factor `interp/decim`.
    pub fn resampling<InputType, OutputType>(
        interp: usize,
        decim: usize,
    ) -> TagFir<InputType, OutputType, f32, PolyphaseResamplingFir<InputType, OutputType, Vec<f32>>>
    where
        InputType: CpuSample,
        OutputType: CpuSample,
        PolyphaseResamplingFir<InputType, OutputType, Vec<f32>>: Filter<InputType, OutputType, f32>,
    {
        let d = gcd(interp, decim);
        let interp = interp / d;
        let decim = decim / d;
        let taps = firdes::kaiser::multirate::<f32>(interp, decim, 12, 0.0001);
        TagFirBuilder::resampling_with_taps::<InputType, OutputType, _>(interp, decim, taps)
    }

    /// Create a new rationally resampling FIR filter with taps.
    pub fn resampling_with_taps<InputType, OutputType, TapsType>(
        interp: usize,
        decim: usize,
        taps: TapsType,
    ) -> TagFir<
        InputType,
        OutputType,
        TapsType::TapType,
        PolyphaseResamplingFir<InputType, OutputType, TapsType>,
    >
    where
        InputType: CpuSample,
        OutputType: CpuSample,
        TapsType: 'static + Taps + Send,
        TapsType::TapType: 'static + Send,
        PolyphaseResamplingFir<InputType, OutputType, TapsType>:
            Filter<InputType, OutputType, TapsType::TapType>,
    {
        TagFir::<
            InputType,
            OutputType,
            TapsType::TapType,
            PolyphaseResamplingFir<InputType, OutputType, TapsType>,
        >::new(
            PolyphaseResamplingFir::new(interp, decim, taps),
            interp,
            decim,
        )
    }
}
