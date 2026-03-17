use anyhow::Result;
use fsdr_cli::blocks::kiss_file_source::KissFileSource;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use std::sync::{Arc, Mutex};

mod test_sink {
    use futuresdr::prelude::*;
    use futuresdr::runtime::Pmt;
    use std::sync::{Arc, Mutex};

    #[derive(Block)]
    #[message_inputs(in_port)]
    pub struct TestMessageSink {
        messages: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl TestMessageSink {
        pub fn new(messages: Arc<Mutex<Vec<Vec<u8>>>>) -> Self {
            Self { messages }
        }

        async fn in_port(
            &mut self,
            io: &mut WorkIo,
            _mio: &mut MessageOutputs,
            _meta: &mut BlockMeta,
            p: Pmt,
        ) -> Result<Pmt> {
            match p {
                Pmt::Blob(bytes) => {
                    self.messages.lock().unwrap().push(bytes);
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
    impl Kernel for TestMessageSink {}
}

use test_sink::TestMessageSink;

#[test]
fn test_kiss_file_source() -> Result<()> {
    // 1. Path to the KISS file in the repository
    let filename = "tests/test.kiss";

    // 2. Set up FutureSDR flowgraph
    let mut fg = Flowgraph::new();
    let src = KissFileSource::new(filename);

    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let sink = TestMessageSink::new(received_messages.clone());

    let src_id = fg.add_block(src);
    let sink_id = fg.add_block(sink);
    fg.connect_message(src_id, "output", sink_id, "in_port")?;

    // 3. Run flowgraph
    Runtime::new().run(fg)?;

    // 4. Assert
    let msgs = received_messages.lock().unwrap();
    assert_eq!(msgs.len(), 2);
    
    // Frame 1 decoded: 00 AA BB
    assert_eq!(msgs[0], vec![0x00, 0xAA, 0xBB]);
    
    // Frame 2 decoded: 00 CC C0 DD
    assert_eq!(msgs[1], vec![0x00, 0xCC, 0xC0, 0xDD]);

    Ok(())
}
