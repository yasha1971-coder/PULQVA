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

    if args.len() != 17 {
        process::exit(2);
    }

    let expected = [
        (0, "-hide_banner"),
        (1, "-loglevel"),
        (2, "error"),
        (3, "-nostdin"),
        (4, "-y"),
        (5, "-protocol_whitelist"),
        (6, "file"),
        (7, "-i"),
        (9, "-map"),
        (10, "0"),
        (11, "-dn"),
        (12, "-c"),
        (13, "copy"),
        (14, "-f"),
    ];

    for (index, value) in expected {
        if args[index].as_os_str() != OsStr::new(value) {
            process::exit(3);
        }
    }

    if args[15].as_os_str() != OsStr::new("mp4")
        && args[15].as_os_str() != OsStr::new("matroska")
    {
        process::exit(4);
    }

    let output = PathBuf::from(&args[16]);
    if let Some(parent) = output.parent() {
        if fs::create_dir_all(parent).is_err() {
            process::exit(5);
        }
    }

    let mut marker_os = output.as_os_str().to_os_string();
    marker_os.push(".pulqva-ffmpeg-fixture.argv");
    let marker = PathBuf::from(marker_os);

    let recorded = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("\n");

    if fs::write(marker, recorded).is_err() {
        process::exit(6);
    }

    thread::sleep(Duration::from_secs(30));
}
