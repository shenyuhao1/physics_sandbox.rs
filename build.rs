fn main() {
    println!("cargo:rustc-link-search=native=.");
    println!("cargo:rustc-link-lib=SDL2");
    println!("cargo:rustc-link-lib=SDL2main");
}