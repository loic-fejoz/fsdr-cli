use fsdr_cli::grc::Grc;
use serde_yaml::{self};

#[test]
pub fn load_realpart_cf_grc() {
    let f = std::fs::File::open("tests/realpart_cf.grc").expect("Could not open file.");
    let grc: Grc = serde_yaml::from_reader(f).expect("Could not read values.");
    println!("{grc:?}");
}
