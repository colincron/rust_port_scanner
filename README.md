# Rust_Port_Scanner

A minimal TCP connect scanner with banner grabbing, written in Rust. Give it a target (single IP or range) and ports (single port or range), and it prints a service banner for each open port.

## How it works

1. Resolves the target IPs and ports from command-line arguments.
2. For each host and port, attempts a full TCP connect with a 200 ms timeout.
3. On an open port, grabs a banner:
   - Reads immediately for up to 3 seconds. This catches protocols that speak first (SSH, FTP, SMTP, etc.).
   - If nothing arrives, sends `HEAD / HTTP/1.0\r\n\r\n` and reads the response. This catches HTTP-style services that wait for the client.
   - If the connection closes with no data, reports the error.

## Requirements

- Rust 1.88 or newer (edition 2024; the code uses `if let` chains).
- `clap` (with the `derive` feature).

## Build

```bash
cargo build --release
```

## Usage

Exactly one IP option and exactly one port option are required.

| Flag | Long | Value | Description |
|---|---|---|---|
| `-i` | `--ip-addr` | `192.168.8.1` | Single target IP |
| `-r` | `--ip-range` | `192.168.8.1-192.168.8.255` | IP range, `start-end` |
| `-p` | `--port` | `22` | Single port |
| `-o` | `--port-range` | `1-1024` | Port range, `start-end` |

Ranges are end-exclusive: `80-90` scans ports 80 through 89, and the end IP of an IP range is not scanned.

```bash
# One host, one port
./target/release/Rust_Port_Scanner -i 192.168.8.1 -p 22

# One host, ports 1-1023
./target/release/Rust_Port_Scanner -i 192.168.8.1 -o 1-1024

# A range of hosts, one port
./target/release/Rust_Port_Scanner -r 192.168.8.1-192.168.8.255 -p 80
```

## Example output

```
192.168.8.1:22 is OPEN

Banner from 192.168.8.1:22:
SSH-2.0-OpenSSH_9.6

192.168.8.1:80 is OPEN

Banner from 192.168.8.1:80:
HTTP/1.0 200 OK
Server: nginx
...
```

Closed and filtered ports print nothing.

## Configuration

Timeouts are hardcoded. Edit and recompile.

| Setting | Location | Default |
|---|---|---|
| Connect timeout | `timeout` in `main()` | 200 ms |
| Banner connect/read/write timeout | `grab_banner()` | 3 s |

## Limitations

- **Sequential.** Hosts and ports are scanned one at a time. Against mostly filtered (silently dropping) hosts, a /24 across 1023 ports can take around 14 hours in the worst case (254 hosts × 1023 ports × 200 ms). Refused connections return quickly, so LAN scans are usually much faster.
- **No CIDR input.** Targets are a single IP or a `start-end` range.
- **Range ends are exclusive**, so port 65535 can only be scanned with `-p`.
- **IPv4 only**, TCP only, no UDP.
- **Full connect scan**, not SYN. It is noisy and will show up in target logs.
- **Banner is raw.** Output is lossy UTF-8 of the first 1024 bytes. There is no service fingerprinting or TLS handling, so HTTPS ports will usually return garbage or nothing.

## Possible improvements

- Configurable timeout flag
- CIDR input
- Concurrency with `tokio`, `rayon`, or a thread pool
- Optional output to JSON or CSV
- TLS banner grabbing

## Legal

Only scan networks you own or have explicit written authorization to test. Unauthorized scanning may violate computer misuse laws.
