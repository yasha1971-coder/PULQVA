use std::{
    env,
    ffi::OsStr,
    fs,
    io::{Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    path::PathBuf,
    process,
    thread,
    time::Duration,
};

fn main() {
    let mut args = env::args_os().skip(1);

    if args.next().as_deref() != Some(OsStr::new("proxy")) {
        process::exit(2);
    }
    if args.next().as_deref() != Some(OsStr::new("--config")) {
        process::exit(3);
    }
    let Some(config_path) = args.next().map(PathBuf::from) else {
        process::exit(4);
    };
    if args.next().is_some() {
        process::exit(5);
    }

    let config = match fs::read_to_string(config_path) {
        Ok(config) => config,
        Err(_) => process::exit(6),
    };

    let Some(port) = parse_socks_port(&config) else {
        process::exit(7);
    };

    if config.contains("defer_bootstrap = true") {
        thread::sleep(Duration::from_secs(30));
        return;
    }

    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
        Ok(listener) => listener,
        Err(_) => process::exit(8),
    };

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else {
            continue;
        };
        let _ = handle_socks(&mut stream);
    }
}

fn parse_socks_port(config: &str) -> Option<u16> {
    config.lines().find_map(|line| {
        let line = line.trim();
        let value = line.strip_prefix("socks_listen = ")?;
        value.parse().ok()
    })
}

fn handle_socks(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut greeting = [0_u8; 3];
    stream.read_exact(&mut greeting)?;
    if greeting != [0x05, 0x01, 0x00] {
        return Ok(());
    }
    stream.write_all(&[0x05, 0x00])?;

    let mut header = [0_u8; 5];
    stream.read_exact(&mut header)?;
    if header[0..4] != [0x05, 0x01, 0x00, 0x03] {
        return Ok(());
    }

    let mut host = vec![0_u8; header[4] as usize];
    stream.read_exact(&mut host)?;

    let mut port = [0_u8; 2];
    stream.read_exact(&mut port)?;

    stream.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 0])?;
    Ok(())
}
