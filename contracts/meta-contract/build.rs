extern crate protoc_rust;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src")
        .inputs(&["src/response.proto"])
        .include("src")
        .run()
        .expect("Running protoc failed.");
}
