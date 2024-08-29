#[allow(dead_code)]
fn generator() {
    use std::env;
    use std::path::PathBuf;

    #[cfg(target_arch = "x86")]
    {
        env::set_var("LIBCLANG_PATH", "D:/Program Files (x86)/LLVM/bin");
        println!("cargo:rustc-link-search=native=D:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.39.33519/lib/x86");
    }
    #[cfg(target_arch = "x86_64")]
    {
        env::set_var("LIBCLANG_PATH", "D:/Program Files/LLVM/bin");
        println!("cargo:rustc-link-search=native=D:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.39.33519/lib/x64");
    }

    // the head path
    let header_path = r"D:\Program Files (x86)\National Instruments\Shared\CVI\include\Nican.h";

    // bindgen
    let bindings = bindgen::Builder::default()
        .header(header_path)
        // .clang_arg("-D__NC_NOINC_compiler")
        // .clang_arg("-D_NCDEF_NOT_DLL_")
        // .clang_arg("-DBASIC")
        .generate()
        .expect("Unable to generate bindings");

    // Write into directory `OUT_DIR
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("nican.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    generator();

    println!("cargo:rustc-link-arg=/SAFESEH:NO");

    println!("cargo:rustc-link-lib=static=legacy_stdio_definitions");

    println!("cargo:rustc-link-search=native=D:/Program Files (x86)/National Instruments/RT Images/NI-CAN");
    println!("cargo:rustc-link-lib=dylib=nican");

    println!("cargo:rustc-link-search=native=D:/Program Files (x86)/National Instruments/Shared/CVI/extlib/msvc");
    println!("cargo:rustc-link-lib=static=nican");
}
