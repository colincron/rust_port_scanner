extern crate ipnet;

use ipnet::Ipv4Net;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::{SocketAddr, TcpStream};
use std::ops::Range;
use std::time::Duration;

fn grab_banner(ip: Ipv4Addr, port: u16) -> Result<String, std::io::Error> {
    // 1. Parse the IP and Port into a SocketAddr
    let socket_addr: SocketAddr = SocketAddrV4::new(ip, port).into();

    // 2. Connect with a strict timeout (e.g., 3 seconds)
    let timeout = Duration::from_secs(3);
    //let addr_string: String = address.to_string();
    //let socket_addr: SocketAddr = addr_string.parse().expect("Invalid address format.");
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

fn main() {
    //let target = Ipv4Addr::new(127, 0, 0, 1);
    let net: Ipv4Net = "192.168.8.0/24".parse().unwrap();
    let timeout = Duration::from_millis(200);
    // let ports_list: Range<u16> = Range {
    //     start: 0,
    //     end: 1024,
    // };
    for ip in net.hosts() {
        let ports_list: Range<u16> = Range {
            start: 0,
            end: 1024,
        };
        for port in ports_list {
            if is_port_open(ip, port, timeout) {
                println!("\n{}:{} is OPEN", ip, port);
                match grab_banner(ip, port) {
                    Ok(banner) => println!("\nBanner from {}:{}:\n{}", ip, port, banner),
                    Err(e) => println!("\nFailed to grab banner from {}:{}\n{}", ip, port, e),
                }
            } else {
                {} // explicitly do nothing, could also delete else block
                // println!("{}:{} is CLOSED", ip, port);
            }
        }
    }
}
