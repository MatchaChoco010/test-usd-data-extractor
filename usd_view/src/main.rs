use usd_data_extractor::*;

fn main() {
    let mut usd_data_extractor = UsdDataExtractor::new("./test-usd/test.usd");

    println!("Extracting USD data... (TimeCode: 1.0)");
    for data in usd_data_extractor.extract(1.0) {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            BridgeData::TimeCodeRange(start, end) => println!("TimeCodeRange: {start} - {end}"),
        }
    }

    println!("Extracting USD data... (TimeCode: 15.0)");
    for data in usd_data_extractor.extract(15.0) {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            _ => {}
        }
    }

    println!("Destroying USD data extractor...");
    for data in usd_data_extractor.destroy() {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            _ => {}
        }
    }
}
