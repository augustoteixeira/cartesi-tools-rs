use bytes::Buf;
use hex_literal::hex;
use sha2::{Digest, Sha512};
use std::path::PathBuf;
use xz2::read::XzDecoder;

const LIBCMT_URL: &str = "https://github.com/cartesi/machine-emulator-tools/releases/download/v0.16.1/libcmt-dev-riscv64-cross-v0.16.1.deb";
const LIBCMT_CHECKSUM: [u8; 64] = hex!("4eafbc8987e1f34d2ec40eb6c90f75ea269041812993227598c88086258189aeef3bdb42790a7504d4f0204b75764aa507002ca5c1433566382f5e531ac8901a");

fn main() {
    download_libcmt();
    println!("cargo:rerun-if-changed=build.rs");
}

fn download_libcmt() {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let lib_path = out_path.join("usr/riscv64-linux-gnu/lib/");
    let headers_path = out_path.join("usr/riscv64-linux-gnu/include/libcmt/");

    // get
    let data = reqwest::blocking::get(LIBCMT_URL)
        .expect("error downloading libcmt")
        .bytes()
        .expect("error getting libcmt request body");

    // checksum
    let mut hasher = Sha512::new();
    hasher.update(&data);
    let result = hasher.finalize();
    assert_eq!(result[..], LIBCMT_CHECKSUM, "libcmt checksum failed");

    let mut archive = ar::Archive::new(data.reader());
    let entry = loop {
        if let Some(Ok(entry)) = archive.next_entry() {
            if entry.header().identifier() == "data.tar.xz".as_bytes() {
                break entry;
            }
        } else {
            panic!("file not found")
        }
    };

    // extract
    let xz = XzDecoder::new(entry);
    let mut inner_archive = tar::Archive::new(xz);
    inner_archive
        .unpack(&out_path)
        .expect("failed to unpack libcmt");

    //     let mut found_lib = false;
    //     let mut found_headers = false;
    //     for file in inner_archive.entries().unwrap() {
    //         let mut f = file.expect("error opening file");
    //         let path = f.header().path().expect("could not get archive path");

    //         let libcmt_expected_path = "./usr/riscv64-linux-gnu/lib/libcmt.a";
    //         let header_expected_path = "./usr/riscv64-linux-gnu/include/";

    //         if path.to_str().expect("invalid utf8 string") == libcmt_expected_path {
    //             f.unpack(out_path.join("libcmt.a"))
    //                 .expect("could not copy libcmt.a");

    //             found_lib = true;
    //         } else if path.to_str().expect("invalid utf8 string") == header_expected_path {
    //             f.unpack(headers_path.clone())
    //                 .expect("could not copy headers");

    //             found_headers = true;
    //         }
    //     }

    //     if !found_lib {
    //         panic!("could not find libcmt.a");
    //     }
    //     if !found_headers {
    //         panic!("could not find headers");
    //     }

    // compiler flags
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=cmt");

    // let libcmt_dir_path = PathBuf::from("./machine-emulator-tools/sys-utils/libcmt/src/")
    //     .canonicalize()
    //     .expect("cannot canonicalize path");

    let libcmt_bindings = bindgen::Builder::default()
        .header(headers_path.join("rollup.h").to_str().unwrap())
        .generate()
        .expect("Unable to generate libcmt bindings");

    libcmt_bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write libcmt bindings");
}
