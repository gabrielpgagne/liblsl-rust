use std::env;

fn main() {
    // TODO: find out if liblsl already present on system and usable (if so, link to that instead)
    // println!("cargo:warning={}", "rebuilding...");
    build_liblsl();
}

// Build the liblsl library from source using cmake
fn build_liblsl() {
    let target = env::var("TARGET").unwrap();

    // build with cmake
    let mut cfg = cmake::Config::new("liblsl");
    cfg.define("LSL_NO_FANCY_LIBNAME", "ON")
        .define("LSL_BUILD_STATIC", "ON");
    if target.contains("msvc") {
        // override some C/CXX flags that the cmake crate splices in on Windows
        // (these cause the build to fail)...
        // * /EHsc sets the correct exception handling mode
        // * /GR enables RTTI
        // * /MD links in the msvcrt as a DLL instead of statically
        let cxx_args = " /nologo /EHsc /MD /GR";
        cfg.define("WIN32", "1")
            .define("_WINDOWS", "1")
            .define("CMAKE_C_FLAGS", cxx_args)
            .define("CMAKE_CXX_FLAGS", cxx_args)
            .define("CMAKE_C_FLAGS_DEBUG", cxx_args)
            .define("CMAKE_CXX_FLAGS_DEBUG", cxx_args)
            .define("CMAKE_C_FLAGS_RELEASE", cxx_args)
            .define("CMAKE_CXX_FLAGS_RELEASE", cxx_args);
    } else if target.contains("android") {
        let ndk = env::var("ANDROID_NDK").expect("ANDROID_NDK_HOME not set");
        let host_triplet = env::var("HOST").expect("HOST not set");
        let host_triplet = host_triplet.split('-');

        let os = host_triplet.clone().nth(2).unwrap();
        let arch = host_triplet.clone().nth(0).unwrap();
        let ndk_toolchain = format!("{}/toolchains/llvm/prebuilt/{}-{}/bin/", ndk, os, arch);

        let current_path = env::var_os("PATH").unwrap_or_default();
        let mut paths = env::split_paths(&current_path).collect::<Vec<_>>();
        paths.push(ndk_toolchain.into());
        let new_path = env::join_paths(paths).expect("Failed to join paths");
        env::set_var("PATH", new_path);

        cfg.define("NDK_PROC_aarch64_ABI", "arm64-v8a")
            .define("ANDROID_NDK", ndk);
    }
    let install_dir = cfg.build();

    // emit link directives
    let libdir = install_dir.join("lib");
    let libname = "lsl";
    println!(
        "cargo:rustc-link-search=native={}",
        libdir.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static={}", libname);

    // make sure we also link some additional libs
    if target.contains("linux") && !target.contains("android") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("windows") {
        // TODO: this is a shortcoming in the current cmake file, which should be
        //       linking in this library (once this is fixed, we should remove this)
        println!("cargo:rustc-link-lib=dylib=bcrypt");
    } else {
        println!("cargo:rustc-link-lib=dylib=c++");
    }
}
