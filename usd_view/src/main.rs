use usd_data_extractor::*;

fn main() {
    let mut usd_data_extractor = UsdDataExtractor::new("./test-usd/test.usd");

    println!("Extracting USD data... (TimeCode: 1.0)");
    let diff = usd_data_extractor.extract(1.0);
    for data in diff {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            BridgeData::TimeCodeRange(start, end) => println!("TimeCodeRange: {start} - {end}"),
        }
    }

    println!("Extracting USD data... (TimeCode: 15.0)");
    let diff = usd_data_extractor.extract(15.0);
    for data in diff {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            _ => {}
        }
    }

    println!("Destroying USD data extractor...");
    let diff = usd_data_extractor.destroy();
    for data in diff {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            _ => {}
        }
    }
}
