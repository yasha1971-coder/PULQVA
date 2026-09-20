use std::{
    env,
    fs,
    path::PathBuf,
    process,
    thread,
    time::Duration,
};

fn main() {
    let mut args = env::args_os().skip(1);

    if args.next().as_deref() != Some(std::ffi::OsStr::new("proxy")) {
        process::exit(2);
    }

    if args.next().as_deref() != Some(std::ffi::OsStr::new("--config")) {
        process::exit(3);
    }

    let Some(config_path) = args.next().map(PathBuf::from) else {
        process::exit(4);
    };

    if args.next().is_some() {
        process::exit(5);
    }

    let config = match fs::read_to_string(&config_path) {
        Ok(config) => config,
        Err(_) => process::exit(6),
    };

    if !config.contains("defer_bootstrap = true") {
        process::exit(7);
    }

    let marker = config_path.with_extension("fixture-ran");
    if fs::write(marker, b"prepared-only direct child launch").is_err() {
        process::exit(8);
    }

    thread::sleep(Duration::from_secs(30));
}
