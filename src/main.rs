use clap::{ArgGroup, Parser};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::{SocketAddr, TcpStream};
use std::ops::RangeInclusive;
use std::time::Duration;

fn clean_banner(raw: &[u8]) -> String {
    // 1. lossy-convert raw to a string (bind it to a variable)
    let banner_input = String::from_utf8_lossy(raw).to_string();
    // 2. take the first line
    let first_line = banner_input.lines().next().unwrap_or("");
    // 3. escape it
    let sanitized_banner = first_line.escape_debug().to_string();
    // 4. return an owned String
    sanitized_banner
}

fn parse_millis(s: &str) -> Result<Duration, std::num::ParseIntError> {
    s.parse::<u64>().map(|m| Duration::from_millis(m))
}
fn parse_port_range(s: &str) -> Result<RangeInclusive<u16>, String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Range must be in the format 'start-end' (e.g., 80-90)".to_string());
    }
    let start = parts[0].parse::<u16>().map_err(|e| e.to_string())?;
    let end = parts[1].parse::<u16>().map_err(|e| e.to_string())?;
    Ok(start..=end)
}

fn parse_ip_range(s: &str) -> Result<RangeInclusive<Ipv4Addr>, String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err(
            "Range must be in the format 'start-end' (e.g., 192.168.1.1-192.168.1.255)".to_string(),
        );
    }
    let start = parts[0].parse::<Ipv4Addr>().map_err(|e| e.to_string())?;
    let end = parts[1].parse::<Ipv4Addr>().map_err(|e| e.to_string())?;
    Ok(start..=end)
}

fn grab_banner(ip: Ipv4Addr, port: u16, timeout: Duration) -> Result<String, std::io::Error> {
    // 1. Parse the IP and Port into a SocketAddr
    let socket_addr: SocketAddr = SocketAddrV4::new(ip, port).into();

    let mut stream = TcpStream::connect_timeout(&socket_addr, timeout)?;

    // 3. Set read/write timeouts so the function doesn't hang indefinitely
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let mut buffer = [0; 1024];

    // 4. Try reading immediately (for protocols that greet first like SSH/FTP)
    if let Ok(bytes_read) = stream.read(&mut buffer)
        && bytes_read > 0
    {
        return Ok(clean_banner(&buffer[0..bytes_read]));
    }

    // 5. Fallback: Send a basic payload if the service waits for client input (like HTTP)
    let _ = stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n");
    let _ = stream.flush();

    // 6. Read the response after the payload
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read > 0 {
        Ok(clean_banner(&buffer[..bytes_read]))
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "Connection closed with no data",
        ))
    }
}
fn is_port_open(ip: Ipv4Addr, port: u16, timeout: Duration) -> bool {
    let addr = SocketAddrV4::new(ip, port);
    TcpStream::connect_timeout(&addr.into(), timeout).is_ok()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// Ensure the user provides exactly one IP option and one Port option
#[command(group(
    ArgGroup::new("ip_source")
        .required(true)
        .args(["ip_addr", "ip_range"]),
))]
#[command(group(
    ArgGroup::new("port_source")
        .required(true)
        .args(["port", "port_range"]),
))]
struct Args {
    #[arg(short = 'i', long)]
    ip_addr: Option<Ipv4Addr>,

    #[arg(short = 'r', long, value_parser = parse_ip_range)]
    ip_range: Option<RangeInclusive<Ipv4Addr>>,

    #[arg(short = 'p', long)]
    port: Option<u16>,

    #[arg(short = 'o', long, value_parser = parse_port_range)]
    port_range: Option<RangeInclusive<u16>>,

    #[arg(short = 't',
        long = "timeout",
        value_name = "MS",
        default_value = "300",
        value_parser = parse_millis,
        help = "Connection timeout in milliseconds"
    )]
    timeout: Duration,
}

fn main() {
    let args = Args::parse();

    // 1. Resolve IPs into a single Iterator type (or Boxed Iterator)
    let ips: Box<dyn Iterator<Item = Ipv4Addr>> = if let Some(single_ip) = args.ip_addr {
        Box::new(std::iter::once(single_ip))
    } else if let Some(range) = args.ip_range {
        Box::new(range)
    } else {
        unreachable!()
    };

    // 2. Resolve Ports into a Vec so you can clone/iterate over it repeatedly
    let ports: Vec<u16> = if let Some(single_port) = args.port {
        vec![single_port]
    } else if let Some(range) = args.port_range {
        range.collect()
    } else {
        unreachable!()
    };

    let timeout = args.timeout;

    // 3. Execution loops remain clean and agnostic of how the input was provided
    for ip in ips {
        for &port in &ports {
            if is_port_open(ip, port, timeout) {
                println!("\n{}:{} is OPEN", ip, port);
                match grab_banner(ip, port, timeout) {
                    Ok(banner) => println!("\nBanner from {}:{}:\n{}", ip, port, banner),
                    Err(e) => println!("\nFailed to grab banner from {}:{}\n{}", ip, port, e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_first_line() {
        let raw = b"SSH-2.0-dropbear\r\n\x00\x00\x01\xff\xfekex-algorithms";
        assert_eq!(clean_banner(raw), "SSH-2.0-dropbear");
    }

    #[test]
    fn escapes_control_chars() {
        let out = clean_banner(b"\x1b[31mred");
        assert!(!out.contains('\x1b'));
    }

    #[test]
    fn empty_input_is_empty_string() {
        assert_eq!(clean_banner(b""), "");
    }
}
