use usd_data_extractor::*;

fn main() {
    let mut usd_data_extractor = UsdDataExtractor::new("./test-usd/test.usd");

    println!("Extracting USD data...");
    usd_data_extractor.show_data();

    println!("Destroying USD data extractor...");
    usd_data_extractor.destroy();
}
