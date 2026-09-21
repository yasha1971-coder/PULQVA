use std::{env, ffi::OsStr, fs, path::PathBuf, process};

fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();

    if args.len() != 6
        || args[0].as_os_str() != OsStr::new("--ignore-config")
        || args[1].as_os_str() != OsStr::new("--proxy")
        || !args[2].to_string_lossy().starts_with("socks5h://")
        || args[3].as_os_str() != OsStr::new("--paths")
    {
        process::exit(2);
    }

    let source = args[5].to_string_lossy();
    if source.ends_with("/fail") {
        process::exit(9);
    }

    let output_root = PathBuf::from(&args[4]);
    if fs::create_dir_all(&output_root).is_err() {
        process::exit(3);
    }

    if fs::write(output_root.join("artifact.bin"), b"PULQVA").is_err() {
        process::exit(4);
    }
}
