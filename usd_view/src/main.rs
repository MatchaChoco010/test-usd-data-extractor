use usd_data_extractor::*;

fn show_data(data: BridgeData) {
    match data {
        BridgeData::Message(s) => println!("{}", s),
        BridgeData::TimeCodeRange(start, end) => println!("TimeCodeRange: {start} - {end}"),
        BridgeData::CreateMesh(path) => println!("{path} [CreateMesh]"),
        BridgeData::TransformMatrix(path, matrix) => {
            println!("{path} [TransformMatrix]");
            for r in 0..4 {
                print!("    ");
                for c in 0..4 {
                    print!("{:+6.4} ", matrix[r * 4 + c]);
                }
                println!();
            }
        }
        BridgeData::Points(path, data, interpolation) => {
            println!(
                "{} [Points], len: {}, interpolation: {:?}",
                path,
                data.len() / 3,
                interpolation
            );
            print!("    ");
            for i in 0..9.min(data.len()) {
                if i % 3 == 0 {
                    print!("(");
                }
                print!("{:+6.4} ", data[i]);
                if i % 3 == 2 {
                    print!("), ");
                }
            }
            println!("...");
        }
        BridgeData::Normals(path, data, interpolation) => {
            println!(
                "{} [Normals], len: {}, interpolation: {:?}",
                path,
                data.len() / 3,
                interpolation
            );
            print!("    ");
            for i in 0..9.min(data.len()) {
                if i % 3 == 0 {
                    print!("(");
                }
                print!("{:+6.4} ", data[i]);
                if i % 3 == 2 {
                    print!("), ");
                }
            }
            println!("...");
        }
        BridgeData::Uvs(path, data, interpolation) => {
            println!(
                "{} [UVs], len: {}, interpolation: {:?}",
                path,
                data.len() / 2,
                interpolation
            );
            print!("    ");
            for i in 0..6.min(data.len()) {
                if i % 2 == 0 {
                    print!("(");
                }
                print!("{:+6.4} ", data[i]);
                if i % 2 == 1 {
                    print!("), ");
                }
            }
            println!("...");
        }
        BridgeData::Indices(path, data) => {
            println!("{} [Indices], len: {}", path, data.len());
            print!("    ");
            for i in 0..6.min(data.len()) {
                print!("{}, ", data[i]);
            }
            println!("...");
        }
        BridgeData::DestroyMesh(path) => println!("{path} [DestroyMesh]"),
    }
}

fn main() {
    let mut usd_data_extractor = UsdDataExtractor::new("./test-usd/test.usd");

    println!("Extracting USD data... (TimeCode: 1.0)");
    let diff = usd_data_extractor.extract(1.0);
    for data in diff {
        show_data(data);
    }

    println!("Extracting USD data... (TimeCode: 15.0)");
    let diff = usd_data_extractor.extract(15.0);
    for data in diff {
        show_data(data);
    }

    println!("Destroying USD data extractor...");
    let diff = usd_data_extractor.destroy();
    for data in diff {
        show_data(data);
    }
}
