# fsdr-cli

A command line interface based on [FutureSDR](http://www.futuresdr.org) meant to be
* a line-for-line replacement of [csdr](https://github.com/jketterl/csdr) ([original](https://github.com/ha7ilm/csdr)),
* yet able to pack pipelined command in one unique-flowgraph,
* able to launch simple GNU Radio Companion flowgraph within FutureSDR runtime,
* a testing tool for [FutureSDR community blocks](https://github.com/FutureSDR/fsdr-blocks)

## WFM decoding

In the [csdr documentation about WFM demodulation](https://github.com/jketterl/csdr#demodulate-wfm), one can find the following command line:
```bash
rtl_sdr -s 240000 -f 89500000 -g 20 - | csdr convert_u8_f | csdr fmdemod_quadri_cf | csdr fractional_decimator_ff 5 | csdr deemphasis_wfm_ff 48000 50e-6 | csdr convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -
```

Does it works with `fsdr-cli`? For sure! Just replace every `csdr` with `fsdr-cli`:
```bash
rtl_sdr -s 240000 -f 89500000 -g 20 - | fsdr-cli convert_u8_f | fsdr-cli fmdemod_quadri_cf | fsdr-cli fractional_decimator_ff 5 | fsdr-cli deemphasis_wfm_ff 48000 50e-6 | fsdr-cli convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -
```

But is it the most efficient way? No. One can benefit from FutureSDR efficient scheduling. You just need to transform it as such:

```bash
rtl_sdr -s 240000 -f 89500000 -g 20 - | fsdr-cli csdr convert_u8_f \| convert_ff_c \| fmdemod_quadri_cf \| fractional_decimator_ff 5 \| deemphasis_wfm_ff 48000 50e-6 \| convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -
```

Bascially, it is just a matter of escaping pipe `|` so that your shell does not interpret them, and asking `fsdr-cli` to interpret the command line as a multi-block of `csdr` command. There is also one newly inserted block `convert_ff_c` due to otherwise ill-typed workflow.

But maybe you are more used to [GNURadio](https://www.gnuradio.org/) and would have come with the following workflow [chain1.grc](tests/chain1.grc):
![](chain1.png)

Well, it also works:

```bash
fsdr-cli grc tests/chain1.grc
```

All would run fine (even keeping the blocks' name):

```
$ ./target/release/fsdr-cli grc tests/chain1.grc 
$ ./target/debug/fsdr-cli grc tests/chain1.grc 
FutureSDR: DEBUG - in run_flowgraph
FutureSDR: DEBUG - Listening on 127.0.0.1:1337
FutureSDR: DEBUG - connect stream io
FutureSDR: DEBUG - connect message io
FutureSDR: DEBUG - init blocks
FutureSDR: DEBUG - wait for blocks init
FutureSDR: DEBUG - running blocks
FutureSDR: DEBUG - blocks_file_source_0 terminating 
FutureSDR: DEBUG - blocks_char_to_float_0 terminating 
FutureSDR: DEBUG - blocks_deinterleave_0 terminating 
FutureSDR: DEBUG - blocks_float_to_complex_0 terminating 
FutureSDR: DEBUG - analog_quadrature_demod_cf_0 terminating 
FutureSDR: DEBUG - rational_resampler_xxx_0 terminating 
FutureSDR: DEBUG - analog_fm_deemph_0 terminating 
FutureSDR: DEBUG - blocks_float_to_short_0 terminating 
FutureSDR: DEBUG - blocks_file_sink_0 terminating 
FutureSDR: DEBUG - blocks_throttle_0 terminating 
FutureSDR: DEBUG - audio_sink_0 terminating
```

## TODO

Some much more to experiment with! [Just come to help](CONTRIBUTING.md). ;-)