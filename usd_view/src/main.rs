use usd_data_extractor::*;

fn main() {
    println!("Hello, world!");
    let mut usd_data_extractor = UsdDataExtractor::new("./test.usd");
    usd_data_extractor.show_data();
}
