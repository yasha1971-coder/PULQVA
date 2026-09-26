//! Fixed public CI destinations only. No host DNS, TLS or HTTP requests.
use std::{io::{Read, Write}, net::{Ipv4Addr, SocketAddr, TcpStream}, time::{Duration, Instant}};

pub(super) fn report(deadline: Instant) {
    for host in ["upload.wikimedia.org", "example.com"] {
        if Instant::now() >= deadline {
            eprintln!("PULQVA_MEDIA_SOCKS_PROBE host={host} result=budget-exhausted");
            break;
        }
        let start = Instant::now();
        let limit = deadline.min(start + Duration::from_secs(5));
        let result = probe(host, limit);
        eprintln!("PULQVA_MEDIA_SOCKS_PROBE host={host} elapsed_ms={} result={result:?}", start.elapsed().as_millis());
    }
}

fn remaining(deadline: Instant) -> Result<Duration, &'static str> {
    deadline.checked_duration_since(Instant::now()).filter(|d| !d.is_zero()).ok_or("deadline")
}
fn read_bytes(stream: &mut TcpStream, bytes: &mut [u8], deadline: Instant) -> Result<(), &'static str> {
    // Reset against one deadline per byte, so partial reads cannot renew budget.
    for byte in bytes {
        stream.set_read_timeout(Some(remaining(deadline)?)).map_err(|_| "read-timeout-setting")?;
        stream.read_exact(std::slice::from_mut(byte)).map_err(|_| "read-failed-or-timeout")?;
    }
    Ok(())
}
fn write_bytes(stream: &mut TcpStream, bytes: &[u8], deadline: Instant) -> Result<(), &'static str> {
    for byte in bytes {
        stream.set_write_timeout(Some(remaining(deadline)?)).map_err(|_| "write-timeout-setting")?;
        stream.write_all(std::slice::from_ref(byte)).map_err(|_| "write-failed-or-timeout")?;
    }
    Ok(())
}
fn request(host: &str) -> Result<Vec<u8>, &'static str> {
    if !matches!(host, "upload.wikimedia.org" | "example.com") { return Err("non-fixture-host"); }
    let mut bytes = vec![5, 1, 0, 3, host.len() as u8];
    bytes.extend_from_slice(host.as_bytes());
    bytes.extend_from_slice(&443u16.to_be_bytes());
    Ok(bytes)
}
fn reply_code(header: [u8; 4]) -> Result<u8, &'static str> {
    if header[0] != 5 || header[2] != 0 || !matches!(header[3], 1 | 3 | 4) {
        return Err("invalid-reply-header");
    }
    Ok(header[1])
}
fn probe(host: &str, deadline: Instant) -> Result<u8, &'static str> {
    let request = request(host)?;
    // Numeric loopback only; hostname travels in SOCKS ATYP=3 to Arti.
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, 19050));
    let mut stream = TcpStream::connect_timeout(&address, remaining(deadline)?)
        .map_err(|_| "local-socks-connect-failed")?;
    write_bytes(&mut stream, &[5, 1, 0], deadline)?;
    let mut greeting = [0; 2];
    read_bytes(&mut stream, &mut greeting, deadline)?;
    if greeting != [5, 0] { return Err("socks-auth-negotiation-failed"); }
    write_bytes(&mut stream, &request, deadline)?;
    let mut header = [0; 4];
    read_bytes(&mut stream, &mut header, deadline)?;
    let code = reply_code(header)?;
    // A failure REP is useful even if Arti immediately closes the connection.
    if code != 0 { return Ok(code); }
    let length = match header[3] {
        1 => 4, 4 => 16,
        3 => { let mut n = [0]; read_bytes(&mut stream, &mut n, deadline)?; n[0] as usize },
        _ => unreachable!(),
    };
    read_bytes(&mut stream, &mut vec![0; length + 2], deadline)?;
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_fixed_remote_dns_requests_are_allowed() {
        let bytes = request("upload.wikimedia.org").unwrap();
        assert_eq!(&bytes[..5], &[5, 1, 0, 3, 20]);
        assert_eq!(&bytes[5..25], b"upload.wikimedia.org");
        assert_eq!(&bytes[25..], &443u16.to_be_bytes());
        assert!(request("localhost").is_err());
        assert_eq!(reply_code([5, 1, 0, 1]), Ok(1));
        assert!(reply_code([4, 0, 0, 1]).is_err());
        assert!(reply_code([5, 0, 1, 1]).is_err());
    }
    #[test]
    fn expired_budget_performs_no_probe() {
        assert_eq!(probe("example.com", Instant::now() - Duration::from_secs(1)), Err("deadline"));
    }
}
