use std::path::Path;

fn main() {
    let root_path = std::env::current_dir().unwrap();
    let lib_path = root_path.join("src/c/lib/nodegraph/");
    println!("cargo:rustc-env=LD_LIBRARY_PATH={:?}:$LD_LIBRARY_PATH", lib_path);
}