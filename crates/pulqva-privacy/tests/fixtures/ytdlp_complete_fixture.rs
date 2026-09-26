use std::{env, ffi::OsStr, fs, path::PathBuf, process};

fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.get(3).map(|x| x.as_os_str()) != Some(OsStr::new("--no-plugin-dirs"))
        || args.get(4).map(|x| x.as_os_str()) != Some(OsStr::new("--no-js-runtimes"))
        || args.get(5).map(|x| x.as_os_str()) != Some(OsStr::new("--no-remote-components"))
    { process::exit(10); }

    if args.len() != 9
        || args[0].as_os_str() != OsStr::new("--ignore-config")
        || args[1].as_os_str() != OsStr::new("--proxy")
        || !args[2].to_string_lossy().starts_with("socks5h://")
        || args[6].as_os_str() != OsStr::new("--paths")
    {
        process::exit(2);
    }

    let source = args[8].to_string_lossy();
    if source.ends_with("/fail") {
        process::exit(9);
    }

    let output_root = PathBuf::from(&args[7]);
    if fs::create_dir_all(&output_root).is_err() {
        process::exit(3);
    }

    if fs::write(output_root.join("artifact.bin"), b"PULQVA").is_err() {
        process::exit(4);
    }
}
