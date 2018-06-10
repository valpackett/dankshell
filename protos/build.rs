extern crate wayland_scanner;

use std::env;
use std::path::Path;
use wayland_scanner::{Side, generate_c_code, generate_c_interfaces};

fn gen_proto(name: &str) {
    let protocol_file = format!("xml/{}.xml", name);
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    generate_c_interfaces(&protocol_file, out_dir.join(format!("{}-interfaces.rs", name)));
    if env::var("CARGO_FEATURE_CLIENT").ok().is_some() {
        generate_c_code(&protocol_file, out_dir.join(format!("{}-client.rs", name)), Side::Client);
    }
    if env::var("CARGO_FEATURE_SERVER").ok().is_some() {
        generate_c_code(&protocol_file, out_dir.join(format!("{}-server.rs", name)), Side::Server);
    }
}

fn main() {
    gen_proto("layer-shell-unstable-v1");
    gen_proto("dank-shell-private-api");
}
