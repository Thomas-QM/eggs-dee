extern crate libloading;
extern crate structopt;

use libloading::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
	#[structopt(short = "v", long = "variance", default_value = "8")]
	variance: usize,
    #[structopt(short = "e", long = "encode")]
    encode: bool,
	#[structopt(name = "DECODE")]
	decode: Option<String>
}

fn main() {
    let exe_path = std::env::current_exe().unwrap();
    let exe_path = exe_path.parent().unwrap();

    let mut lib_path = exe_path.to_path_buf();
    lib_path.push("eggs_dee");

    let mut ahk_path = exe_path.to_path_buf();
    ahk_path.push("ahk_wrapper");
    
    let lib = Library::new(lib_path.as_os_str()).expect("eggs_dee.dll/.so could not be loaded!");

    let opt = Opt::from_args();

    unsafe {
        if let Some(decode) = opt.decode {
            if opt.encode {
                let encoder: Symbol<unsafe extern fn(usize, &str) -> String> = lib.get(b"encode").unwrap();
                let encoded = encoder(opt.variance, &decode);

                println!("{}", encoded);
            } else {
                let decoder: Symbol<unsafe extern fn(usize, &str) -> String> = lib.get(b"decode").unwrap();
                let decoded = decoder(opt.variance, &decode);

                println!("{}", decoded);
            }
        } else {
            std::process::Command::new(ahk_path.as_os_str())
                .output().expect("Failed to execute AHK wrapper");
        }
    }
}
