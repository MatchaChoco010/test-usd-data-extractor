use usd_data_extractor::*;

fn main() {
    let mut usd_data_extractor = UsdDataExtractor::new("./test-usd/test.usd");

    println!("Extracting USD data... (TimeCode: 1.0)");
    let diff = usd_data_extractor.extract(1.0);
    for data in diff {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            BridgeData::TimeCodeRange(start, end) => println!("TimeCodeRange: {start} - {end}"),
            BridgeData::TransformMatrix(path, matrix) => {
                println!("TransformMatrix: {path}");
                for r in 0..4 {
                    print!("    ");
                    for c in 0..4 {
                        print!("{:+6.4} ", matrix[r * 4 + c]);
                    }
                    println!();
                }
            }
        }
    }

    println!("Extracting USD data... (TimeCode: 15.0)");
    let diff = usd_data_extractor.extract(15.0);
    for data in diff {
        match data {
            BridgeData::Message(s) => println!("{}", s),
            BridgeData::TransformMatrix(path, matrix) => {
                println!("TransformMatrix: {path}");
                for r in 0..4 {
                    print!("    ");
                    for c in 0..4 {
                        print!("{:+6.4} ", matrix[r * 4 + c]);
                    }
                    println!();
                }
            }
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
