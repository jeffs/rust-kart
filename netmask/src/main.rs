//! Prints the narrowest netmask that's inclusive of all specified netmasks.  Netmasks may be in
//! CIDR notation, or may be literal IP addresses (implicitly requiring all 32 bits).

use std::{
    env,
    error::Error,
    fmt,
    net::{AddrParseError, Ipv4Addr},
    num::ParseIntError,
    str::FromStr,
};

use thiserror::Error;

#[allow(clippy::cast_possible_truncation)]
const IPV4_ADDR_BITS: u8 = Ipv4Addr::BITS as u8;

#[derive(Debug, Error)]
enum ParseNetmaskError {
    #[error(transparent)]
    BadIp(#[from] AddrParseError),
    #[error(transparent)]
    BadLen(#[from] ParseIntError),
}

fn keep_left(ip: Ipv4Addr, len: u8) -> Ipv4Addr {
    Ipv4Addr::from_bits(ip.to_bits() & mask_left(len))
}

/// Returns a value in which the leftmost len bits are 1s, and the remaining 32 - len bits are 0s.
fn mask_left(len: u8) -> u32 {
    let len = u32::from(len);
    assert!(len <= u32::BITS);
    u32::MAX
        - ((1u32.checked_shl(u32::BITS - len))
            .unwrap_or_default()
            .wrapping_sub(1))
}

#[derive(Debug)]
struct Netmask {
    ip: Ipv4Addr,
    /// Number of significant bits.
    len: u8,
}

impl Netmask {
    /// Returns the narrowest netmask allowing a superset of self and other.
    fn ancestor(&self, other: &Netmask) -> Netmask {
        let ours = self.ip.to_bits();
        let theirs = other.ip.to_bits();
        let max_len = self.len.min(other.len);
        let len = (0..max_len)
            .find(|&len| {
                let mask = mask_left(len + 1);
                ours & mask != theirs & mask
            })
            .unwrap_or(max_len);
        Netmask {
            ip: keep_left(self.ip, len),
            len,
        }
    }

    fn bits(&self) -> u32 {
        self.ip.to_bits() & mask_left(self.len)
    }
}

impl fmt::Display for Netmask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.ip, self.len)
    }
}

impl PartialEq for Netmask {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.bits() == other.bits()
    }
}

impl FromStr for Netmask {
    type Err = ParseNetmaskError;

    /// Expects CIDR notation; e.g., 12.34.56.78/24.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip, len) = match s.split_once('/') {
            Some((ip, len)) => (ip, len.parse()?),
            None => (s, IPV4_ADDR_BITS),
        };
        let ip = ip.parse()?;
        let netmask = Netmask {
            ip: keep_left(ip, len),
            len,
        };
        if ip != netmask.ip {
            eprintln!("warning: dropping extra bits from {ip} for {netmask}");
        }
        Ok(netmask)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let masks: Result<Vec<Netmask>, _> = env::args().skip(1).map(|arg| arg.parse()).collect();
    if let Some(ancestor) = masks?.into_iter().reduce(|lhs, rhs| lhs.ancestor(&rhs)) {
        println!("{ancestor}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netmask_ancestor() {
        let lhs: Netmask = "12.34.56.78/20".parse().expect("hard-coded netmask");
        let rhs: Netmask = "12.34.65.78/20".parse().expect("hard-coded netmask");
        let want: Netmask = "12.34.0.0/17".parse().expect("hard-coded netmask");
        assert_eq!(lhs.ancestor(&rhs), want);
    }

    #[test]
    fn test_ancestor_max_len() {
        // Even if two masks are identical, their ancestor is limited to the shorter mask's length.
        let lhs: Netmask = "12.34.56.78/16".parse().expect("hard-coded netmask");
        let rhs: Netmask = "12.34.56.78".parse().expect("hard-coded netmask");
        assert_eq!(lhs.ancestor(&rhs), lhs);
    }

    #[test]
    fn test_ancestor_exclude_first_difference() {
        // The ancestor length should be one _less_ than the index of the first differing bit.
        let lhs = Netmask {
            ip: Ipv4Addr::from_bits(0xff00_0000),
            len: 8,
        };
        let rhs = Netmask {
            ip: Ipv4Addr::from_bits(0xfe00_0000),
            len: 8,
        };
        assert_eq!(lhs.ancestor(&rhs).len, 7);
    }

    #[test]
    fn test_mask_left() {
        assert_eq!(mask_left(0), 0);
        assert_eq!(mask_left(8), 0xff00_0000);
        assert_eq!(mask_left(32), 0xffff_ffff);
    }
}
