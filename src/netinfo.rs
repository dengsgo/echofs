use std::net::IpAddr;

/// Enumerate the machine's non-loopback IP addresses.
///
/// Uses two strategies and merges the results:
/// 1. A UDP "connect" probe to a public address to discover the default-route
///    source IP (no traffic is actually sent).
/// 2. On Unix, `getifaddrs(3)` to enumerate every interface.
///
/// Used to print/display all LAN addresses a wildcard-bound server is reachable
/// on. Loopback addresses are skipped.
pub fn local_ips() -> Vec<IpAddr> {
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
