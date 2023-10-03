fn main() {
    println!("cargo:rustc-link-lib=dylib=nodegraph");
    println!("cargo:rustc-link-search=native=/home/biblbrox/Tesis/robot_controller/src/c/lib/nodegraph");
}