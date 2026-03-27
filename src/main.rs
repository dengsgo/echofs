use clap::Parser;
use echofs::{cli, logging, server};
use cli::Args;
use std::net::IpAddr;

fn get_local_ips() -> Vec<IpAddr> {
    let mut ips = Vec::new();
    if let Ok(ifaces) = std::net::UdpSocket::bind("0.0.0.0:0") {
        // Probe trick: connect to a public address to find the default route IP.
        // This doesn't actually send traffic.
        let _ = ifaces.connect("8.8.8.8:80");
        if let Ok(local_addr) = ifaces.local_addr() {
            ips.push(local_addr.ip());
        }
    }

    // Also enumerate all interfaces via getifaddrs on unix
    #[cfg(unix)]
    {
        use std::ffi::CStr;
        use std::net::{Ipv4Addr, Ipv6Addr};

        unsafe extern "C" {
            fn getifaddrs(ifap: *mut *mut libc::ifaddrs) -> libc::c_int;
            fn freeifaddrs(ifa: *mut libc::ifaddrs);
        }

        unsafe {
            let mut ifap: *mut libc::ifaddrs = std::ptr::null_mut();
            if getifaddrs(&mut ifap) == 0 {
                let mut cursor = ifap;
                while !cursor.is_null() {
                    let ifa = &*cursor;
                    if !ifa.ifa_addr.is_null() {
                        let family = (*ifa.ifa_addr).sa_family as libc::c_int;
                        let name = CStr::from_ptr(ifa.ifa_name).to_string_lossy();
                        // Skip loopback
                        if name != "lo" && name != "lo0" {
                            if family == libc::AF_INET {
                                let addr = &*(ifa.ifa_addr as *const libc::sockaddr_in);
                                let ip = Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));
                                let ip = IpAddr::V4(ip);
                                if !ip.is_loopback() && !ips.contains(&ip) {
                                    ips.push(ip);
                                }
                            } else if family == libc::AF_INET6 {
                                let addr = &*(ifa.ifa_addr as *const libc::sockaddr_in6);
                                let ip = Ipv6Addr::from(addr.sin6_addr.s6_addr);
                                let ip = IpAddr::V6(ip);
                                if !ip.is_loopback() && !ips.contains(&ip) {
                                    ips.push(ip);
                                }
                            }
                        }
                    }
                    cursor = ifa.ifa_next;
                }
                freeifaddrs(ifap);
            }
        }
    }

    ips
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let root = args.root_path();
    let addr = args.bind_addr();

    println!(r#"
  ______
 | ____  \
 |      \_\________      ___      _          ___  ___
 |          \ \ \  |    | __|__ | |_   ___  | __|| __|
 |          | | |  |    | _|/ _|| ' \ / _ \ | _| |__ \
 |          / / /  |    |___\__||_||_|\___/ |_|  |___/
 |_________________|    v{}
"#, env!("CARGO_PKG_VERSION"));
    println!("Serving {} on http://{}", root.display(), addr);

    if args.bind == "0.0.0.0" || args.bind == "::" {
        println!("Available on:");
        println!("  http://127.0.0.1:{}", args.port);
        for ip in get_local_ips() {
            match ip {
                IpAddr::V6(v6) => println!("  http://[{}]:{}", v6, args.port),
                _ => println!("  http://{}:{}", ip, args.port),
            }
        }
    }

    let log_target = logging::LogTarget::from_arg(&args.log);
    let speed_limit = args.speed_limit_bytes();

    if let Some(limit) = speed_limit {
        let display = if limit >= 1024 * 1024 * 1024 {
            format!("{:.1} GB/s", limit as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if limit >= 1024 * 1024 {
            format!("{:.1} MB/s", limit as f64 / (1024.0 * 1024.0))
        } else if limit >= 1024 {
            format!("{:.1} KB/s", limit as f64 / 1024.0)
        } else {
            format!("{} B/s", limit)
        };
        println!("Speed limit: {} per request", display);
    }

    let webdav = !args.no_webdav;

    if args.no_webdav {
        println!("WebDAV: disabled");
    }

    if args.open {
        let url = format!("http://127.0.0.1:{}", args.port);
        let _ = open::that(&url);
    }

    server::run(root, &addr, log_target, args.show_hidden, args.max_depth, speed_limit, webdav).await;
}
