use fsdr_blocks::packet::PatternSearch;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::VectorSink;
use fsdr_blocks::futuresdr::blocks::VectorSinkBuilder;
use fsdr_blocks::futuresdr::blocks::VectorSource;
use fsdr_blocks::futuresdr::macros::connect;
use fsdr_blocks::futuresdr::runtime::Flowgraph;
use fsdr_blocks::futuresdr::runtime::Runtime;

#[test]
fn pattern_search_two_bytes_found() -> Result<()> {
    let mut fg = Flowgraph::new();

    let block_under_test = PatternSearch::<u8>::build(2, vec![0xCC, 0xDD]);

    let orig: Vec<u8> = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x01, 0x02, 0x03];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink = VectorSinkBuilder::<u8>::new().build();

    connect!(fg,
        src > block_under_test > vect_sink;
    );
    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<u8>>(vect_sink).unwrap();
    let v = snk.items();

    assert_eq!(v.len(), 2);

    Ok(())
}

#[test]
fn pattern_search_two_bytes_not_found() -> Result<()> {
    let mut fg = Flowgraph::new();

    let block_under_test = PatternSearch::<u8>::build(2, vec![0xAB, 0xCD]);

    let orig: Vec<u8> = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x01, 0x02, 0x03];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink = VectorSinkBuilder::<u8>::new().build();

    connect!(fg,
        src > block_under_test > vect_sink;
    );
    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<u8>>(vect_sink).unwrap();
    let v = snk.items();

    assert_eq!(v.len(), 0);

    Ok(())
}