use std::{
    env,
    ffi::OsStr,
    fs,
    path::PathBuf,
    process,
    thread,
    time::Duration,
};

fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();

    if args.len() != 6 {
        process::exit(2);
    }
    if args[0].as_os_str() != OsStr::new("--ignore-config") {
        process::exit(3);
    }
    if args[1].as_os_str() != OsStr::new("--proxy") {
        process::exit(4);
    }
    if !args[2].to_string_lossy().starts_with("socks5h://") {
        process::exit(5);
    }
    if args[3].as_os_str() != OsStr::new("--paths") {
        process::exit(6);
    }

    let output_root = PathBuf::from(&args[4]);
    if fs::create_dir_all(&output_root).is_err() {
        process::exit(7);
    }

    let marker = output_root.join("pulqva-ytdlp-fixture.argv");
    let recorded = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("\n");

    if fs::write(marker, recorded).is_err() {
        process::exit(8);
    }

    thread::sleep(Duration::from_secs(30));
}
