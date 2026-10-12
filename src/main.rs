extern crate ipnet;

use clap::Parser;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::{SocketAddr, TcpStream};
use std::ops::Range;
use std::time::Duration;

fn parse_port_range(s: &str) -> Result<Range<u16>, String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Range must be in the format 'start-end' (e.g., 80-90)".to_string());
    }
    let start = parts[0].parse::<u16>().map_err(|e| e.to_string())?;
    let end = parts[1].parse::<u16>().map_err(|e| e.to_string())?;
    Ok(start..end)
}

fn parse_ip_range(s: &str) -> Result<Range<Ipv4Addr>, String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err(
            "Range must be in the format 'start-end' (e.g., 192.168.1.1-192.168.1.255)".to_string(),
        );
    }
    let start = parts[0].parse::<Ipv4Addr>().map_err(|e| e.to_string())?;
    let end = parts[1].parse::<Ipv4Addr>().map_err(|e| e.to_string())?;
    Ok(start..end)
}

fn grab_banner(ip: Ipv4Addr, port: u16) -> Result<String, std::io::Error> {
    // 1. Parse the IP and Port into a SocketAddr
    let socket_addr: SocketAddr = SocketAddrV4::new(ip, port).into();

    // 2. Connect with a strict timeout (e.g., 3 seconds)
    let timeout = Duration::from_secs(3);

    let mut stream = TcpStream::connect_timeout(&socket_addr, timeout)?;

    // 3. Set read/write timeouts so the function doesn't hang indefinitely
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let mut buffer = [0; 1024];

    // 4. Try reading immediately (for protocols that greet first like SSH/FTP)
    if let Ok(bytes_read) = stream.read(&mut buffer)
        && bytes_read > 0
    {
        return Ok(String::from_utf8_lossy(&buffer[..bytes_read]).into_owned());
    }

    // 5. Fallback: Send a basic payload if the service waits for client input (like HTTP)
    let _ = stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n");
    let _ = stream.flush();

    // 6. Read the response after the payload
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read > 0 {
        Ok(String::from_utf8_lossy(&buffer[..bytes_read]).into_owned())
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
struct Args {
    // #[arg(short, long)]
    // ip_addr: IpAddr,
    #[arg(short, long, value_parser = parse_ip_range)]
    ip_range: Range<Ipv4Addr>,

    // port: u16,
    #[arg(short, long, value_parser = parse_port_range)]
    port_range: Range<u16>,
}
fn main() {
    let args = Args::parse();

    let Args {
        //ip_addr,
        ip_range,
        //port,
        port_range,
    } = args;

    //let net: Ipv4Net = "192.168.8.0/24".parse().unwrap();
    let timeout = Duration::from_millis(200);

    for ip in ip_range {
        // let ports_list: Range<u16> = Range {
        //     start: 0,
        //     end: 1024,
        // };
        for port in port_range.clone() {
            if is_port_open(ip, port, timeout) {
                println!("\n{}:{} is OPEN", ip, port);
                match grab_banner(ip, port) {
                    Ok(banner) => println!("\nBanner from {}:{}:\n{}", ip, port, banner),
                    Err(e) => println!("\nFailed to grab banner from {}:{}\n{}", ip, port, e),
                }
            } else {
                {} // explicitly do nothing, could also delete else block
            }
        }
    }
}
