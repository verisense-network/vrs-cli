use std::fs::File;

fn main() {
    let mut rsp = reqwest::blocking::get("https://github.com/verisense-network/verisense/raw/refs/heads/main/metadata/metadata.scale").expect("downloading metadata failed");
    let mut f = File::create("metadata.scale").expect("couldn't create metadata.scale");
    rsp.copy_to(&mut f).expect("couldn't write metadata.scale");
}
