use anyhow::Result;
use fsdr_cli::blocks::pattern_search::PatternSearch;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSource;
use futuresdr::prelude::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

#[test]
fn pattern_search_two_bytes_found() -> Result<()> {
    let mut fg = Flowgraph::new();

    let block_under_test = PatternSearch::<u8>::new(2, vec![0xCC, 0xDD]);
    let orig: Vec<u8> = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x01, 0x02, 0x03];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink = VectorSink::<u8>::new(16);

    connect!(fg,
        src > block_under_test > vect_sink;
    );
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    assert_eq!(v.len(), 2);

    Ok(())
}

#[test]
fn pattern_search_two_bytes_not_found() -> Result<()> {
    let mut fg = Flowgraph::new();

    let block_under_test = PatternSearch::<u8>::new(2, vec![0xAB, 0xCD]);
    let orig: Vec<u8> = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x01, 0x02, 0x03];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink = VectorSink::<u8>::new(16);

    connect!(fg,
        src > block_under_test > vect_sink;
    );
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    assert_eq!(v.len(), 0);

    Ok(())
}

#[test]
fn pattern_search_three_bytes_found() -> Result<()> {
    let mut fg = Flowgraph::new();

    let block_under_test = PatternSearch::<u8>::new(3, vec![0xCC, 0xDD, 0xCC]);
    let orig: Vec<u8> = vec![
        0xAA, 0xBB, 0xCC, 0xDD, 0xCC, 0xEE, 0xFF, 0x00, 0xCC, 0x01, 0xCC, 0xDD, 0x02, 0x03, 0x04,
    ];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink = VectorSink::<u8>::new(16);

    connect!(fg,
        src > block_under_test > vect_sink;
    );
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    assert_eq!(v.len(), 3);
    assert_eq!(v[0], 0xEE);
    assert_eq!(v[1], 0xFF);
    assert_eq!(v[2], 0x00);

    Ok(())
}
