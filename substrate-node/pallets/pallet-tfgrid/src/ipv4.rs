#![no_std]

/// List of all private subnets, this includes the multicast range (i.e. list of the non public
/// unicast subnets).
///
/// An IP is considered public if it is not part of a private range. Private ranges are considered
/// to be:
///
/// - 0.0.0.0/8 - current network
/// - 10.0.0.0/8 - private network
/// - 100.64.0.0/10 - carrier grade NAT
/// - 127.0.0.0/8 - loopback addresses
/// - 169.254.0.0/16 - link-local addresses
/// - 172.16.0.0/12 - private network
/// - 192.0.0.0/24 - IETF protocol assignments
/// - 192.0.2.0/24 - TEST-NET-1, documentation and examples
/// - 192.88.99.0/24 - Reserved, formerly 6 to 4 relay
/// - 192.168.0.0/16 - private network
/// - 198.18.0.0/15 - private -network - benchmark testing inter-networking
/// - 198.51.100.0/24 - TEST-NET-2, documentation and examples
/// - 203.0.113.0/24 - TEST-NET-3, documentation and examples
/// - 224.0.0.0/14 - Multicast range
/// - 233.252.0.0/24 - Multicast testnet, documentation and examples
/// - 240.0.0.0/4 - Reserved for future use
/// - 255.255.255.255/32 - Limited broadcast range
const PRIVATE_IP4_SUBNETS: [([u8; 4], u8); 17] = [
    ([0, 0, 0, 0], 8),
    ([10, 0, 0, 0], 8),
    ([100, 64, 0, 0], 10),
    ([127, 0, 0, 0], 8),
    ([169, 254, 0, 0], 16),
    ([172, 16, 0, 0], 12),
    ([192, 0, 0, 0], 24),
    ([192, 0, 2, 0], 24),
    ([192, 88, 99, 0], 24),
    ([192, 168, 0, 0], 16),
    ([198, 18, 0, 0], 15),
    ([198, 51, 100, 0], 24),
    ([203, 0, 113, 0], 24),
    ([224, 0, 0, 0], 4),
    ([233, 252, 0, 0], 24),
    ([240, 0, 0, 0], 4),
    ([255, 255, 255, 255], 32),
];

/// Parses ACII input bytes to an IPv4 address.
pub fn parse_ipv4(input: &[u8]) -> Result<[u8; 4], ()> {
    if input.len() < 7 || input.len() > 15 {
        return Err(());
    }

    let mut sections = 0;
    // See below why this is true
    let mut last_char_was_dot = true;
    let mut octets = [0, 0, 0, 0];
    // Use a u8 accumulator since octests are u8. This way we can use checked operations, and if we
    // have an overflow we know there is an invalid value;
    let mut accumulator: u8 = 0;

    for c in input {
        match c {
            b'0'..=b'9' => {
                // Don't allow leading digits 0.
                // Need to allow single 0 digits though. Instead if doing a look ahead for a '.'
                // character, we fail if accumulator is at 0 and the last char was not a dot.
                // This works because we start with the last_char_was_dot flag set to true.
                if accumulator == 0 && !last_char_was_dot {
                    return Err(());
                }
                accumulator = if let Some(a) = accumulator.checked_mul(10) {
                    a
                } else {
                    return Err(());
                };
                accumulator = if let Some(a) = accumulator.checked_add(c - b'0') {
                    a
                } else {
                    return Err(());
                };

                // clear flag
                last_char_was_dot = false;
            }
            b'.' => {
                if last_char_was_dot {
                    return Err(());
                }

                // set octet value
                octets[sections] = accumulator;

                // advance section
                sections += 1;
                accumulator = 0;

                // shield if there are too many sections
                if sections > 3 {
                    return Err(());
                }

                // set flag
                last_char_was_dot = true;
            }
            _ => return Err(()),
        }
    }

    // At this point the last section can't be saved yet
    octets[sections] = accumulator;

    // Sections must only be 3 here since we have 0 based indexing
    if sections == 3 && !last_char_was_dot {
        Ok(octets)
    } else {
        Err(())
    }
}

/// Parses ASCII input bytes to an IPv4 in CIDR notation.
pub fn parse_ip_cidr(input: &[u8]) -> Result<([u8; 4], u8), ()> {
    // Input can be at most 18 bytes.
    if input.len() > 18 {
        return Err(());
    }

    let sep_pos = if let Some(pos) = input.iter().position(|c| c == &b'/') {
        pos
    } else {
        return Err(());
    };

    let prefix = if let Ok(prefix) = parse_ipv4(&input[..sep_pos]) {
        prefix
    } else {
        return Err(());
    };

    let mask_input = &input[sep_pos + 1..];
    if mask_input.len() > 2 || mask_input.is_empty() {
        return Err(());
    }

    let mut mask: u8 = 0;
    for c in mask_input {
        match c {
            b'0'..=b'9' => {
                // only 2 digits max so we can never overflow a u8, no need for checked mul and add.
                mask *= 10;
                mask += c - b'0';
            }
            _ => return Err(()),
        }
    }

    // 32 bits max in an IPv4, also if mask is 0 it must be single digit.
    if mask > 32 || (mask == 0 && mask_input.len() > 1) {
        return Err(());
    }

    Ok((prefix, mask))
}

/// Parses a subnet in CIDR notation. This parses the IP and mask, and additionally validates that
/// all masked bits are set to 0.
fn parse_subnet(input: &[u8]) -> Result<([u8; 4], u8), ()> {
    let (prefix, mask) = parse_ip_cidr(input)?;

    // Ensure only part of the prefix is set
    let ip_bits = ip_bytes_to_bits(prefix);
    if ip_bits & get_mask_bits(mask) != ip_bits {
        return Err(());
    }

    Ok((prefix, mask))
}

/// Checks if an IP (in CIDR notation) is a public unicast address.
///
/// # Returns
///
/// This function returns an error variant if parsing the IP CIDR fails. If parsing is successful,
/// an OK variant is return with a boolean indicating the IP is public and a unicast or not.
///
/// Technically an IP can also be anycast, we don't have any way to statically validated that
/// except hard-coding known ones.
fn is_ip_cidr_public_unicast(input: &[u8]) -> Result<bool, ()> {
    // We don't care about the mask here
    let (input_ip, _) = parse_ip_cidr(input)?;
    Ok(is_public_unicast(input_ip))
}

/// Checks if an IP (without CIDR notation) is a public unicast address.
///
/// # Returns
///
/// This function returns an error variant if parsing the IP fails. If parsing is successful,
/// an OK variant is return with a boolean indicating the IP is public and a unicast or not.
///
/// Technically an IP can also be anycast, we don't have any way to statically validated that
/// except hard-coding known ones.
fn is_ip_public_unicast(input: &[u8]) -> Result<bool, ()> {
    let ip = parse_ipv4(input)?;
    Ok(is_public_unicast(ip))
}

/// helper function to validate a parsed IP
fn is_public_unicast(input_ip: [u8; 4]) -> bool {
    for (private_ip, mask) in PRIVATE_IP4_SUBNETS {
        let mask_bits = get_mask_bits(mask);
        // Ensure only part of the prefix is set
        let input_ip_bits: u32 = ip_bytes_to_bits(input_ip);
        let private_ip_bits: u32 = ip_bytes_to_bits(private_ip);
        if input_ip_bits & mask_bits == private_ip_bits {
            return false;
        }
    }

    true
}

/// Check if the given `ip` (in ASCII bytes) is part of the network defined by the `ip_cidr` (ip in
/// CIDR notation in ASCII bytes).
fn is_part_of_cidr(ip_cidr_input: &[u8], ip_input: &[u8]) -> Result<bool, ()> {
    let ip = parse_ipv4(ip_input)?;
    let (ip_cidr, mask) = parse_ip_cidr(ip_cidr_input)?;

    let mask_bits = get_mask_bits(mask);
    let ip_bits = ip_bytes_to_bits(ip);
    let ip_cidr_bits = ip_bytes_to_bits(ip_cidr);

    Ok(ip_bits & mask_bits == ip_cidr_bits & mask_bits)
}

/// Helper to convert a 4 byte array to 32 ipv4 bits
fn ip_bytes_to_bits(input: [u8; 4]) -> u32 {
    (input[0] as u32) << 24 | (input[1] as u32) << 16 | (input[2] as u32) << 8 | (input[3] as u32)
}

/// Converts a mask to the bit representation
fn get_mask_bits(mask: u8) -> u32 {
    // u64 so we don't have overflow on /32
    // TODO: benchmark if looping with shifts is faster
    !((2_u64.pow(32 - mask as u32) - 1) as u32)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_valid_ips() {
        assert_eq!(super::parse_ipv4("1.1.1.1".as_bytes()), Ok([1, 1, 1, 1]));
        assert_eq!(
            super::parse_ipv4("100.200.10.0".as_bytes()),
            Ok([100, 200, 10, 0])
        );
        assert_eq!(
            super::parse_ipv4("255.255.255.255".as_bytes()),
            Ok([255, 255, 255, 255])
        );
        assert_eq!(super::parse_ipv4("0.0.0.0".as_bytes()), Ok([0, 0, 0, 0]));
    }

    #[test]
    fn reject_invalid_ips() {
        assert!(super::parse_ipv4("1.1.1.1.".as_bytes()).is_err());
        assert!(super::parse_ipv4("1.1.1.1.1".as_bytes()).is_err());
        assert!(super::parse_ipv4("1.1.1.".as_bytes()).is_err());
        assert!(super::parse_ipv4("1.1.1".as_bytes()).is_err());
        assert!(super::parse_ipv4("255.255.255.256".as_bytes()).is_err());
        assert!(super::parse_ipv4("256.255.255.255".as_bytes()).is_err());
        assert!(super::parse_ipv4("1.10.100.1000".as_bytes()).is_err());
        assert!(super::parse_ipv4("1000.100.10.1".as_bytes()).is_err());
        assert!(super::parse_ipv4("af.fe.ff.ac".as_bytes()).is_err());
        assert!(super::parse_ipv4("00.0.0.0".as_bytes()).is_err());
        assert!(super::parse_ipv4("1.01.2.3".as_bytes()).is_err());
        assert!(super::parse_ipv4("1:1:1:1".as_bytes()).is_err());
        assert!(super::parse_ipv4("1.1.2_.3".as_bytes()).is_err());
    }

    #[test]
    fn accept_valid_cidr() {
        assert_eq!(
            super::parse_ip_cidr("1.1.1.1/32".as_bytes()),
            Ok(([1, 1, 1, 1], 32))
        );
        assert_eq!(
            super::parse_ip_cidr("255.255.255.255/32".as_bytes()),
            Ok(([255, 255, 255, 255], 32))
        );
        assert_eq!(
            super::parse_ip_cidr("255.255.255.129/25".as_bytes()),
            Ok(([255, 255, 255, 129], 25))
        );
        assert_eq!(
            super::parse_ip_cidr("128.0.0.0/1".as_bytes()),
            Ok(([128, 0, 0, 0,], 1))
        );
        assert_eq!(
            super::parse_ip_cidr("32.40.50.24/29".as_bytes()),
            Ok(([32, 40, 50, 24], 29))
        );
        assert_eq!(
            super::parse_ip_cidr("10.0.0.1/8".as_bytes()),
            Ok(([10, 0, 0, 1], 8))
        );
    }

    #[test]
    fn reject_invalid_cidr() {
        assert!(super::parse_ip_cidr("1.1.1.1/33".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("255.255.255.255/00".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("1.1.1.1//1".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("50.40.50.23/160".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("1.1.1.1/".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("1.1.1.1/000".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("1.1.1.1".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("1.1.1.1032".as_bytes()).is_err());
        assert!(super::parse_ip_cidr("1.1.1.1/.".as_bytes()).is_err());
    }

    #[test]
    fn accept_valid_subnet() {
        assert_eq!(
            super::parse_subnet("192.168.0.0/16".as_bytes()),
            Ok(([192, 168, 0, 0], 16))
        );
        assert_eq!(
            super::parse_subnet("10.0.0.0/7".as_bytes()),
            Ok(([10, 0, 0, 0], 7))
        );
        assert_eq!(
            super::parse_subnet("10.0.0.0/9".as_bytes()),
            Ok(([10, 0, 0, 0], 9))
        );
    }

    #[test]
    fn reject_invalid_subnet() {
        assert!(super::parse_subnet("10.0.0.0/6".as_bytes()).is_err());
    }

    #[test]
    fn valid_public_ip_cidr() {
        // Technically unicast but yeah
        assert_eq!(
            super::is_ip_cidr_public_unicast("1.1.1.1/32".as_bytes()),
            Ok(true)
        );
        assert_eq!(
            super::is_ip_cidr_public_unicast("10.10.100.254/24".as_bytes()),
            Ok(false)
        );
        assert_eq!(
            super::is_ip_cidr_public_unicast("10.10.100.254/24".as_bytes()),
            Ok(false)
        );
        assert_eq!(
            super::is_ip_cidr_public_unicast("172.10.100.254/24".as_bytes()),
            Ok(true)
        );
        assert_eq!(
            super::is_ip_cidr_public_unicast("172.20.100.254/24".as_bytes()),
            Ok(false)
        );
    }

    #[test]
    fn invalid_public_ip_cidr() {
        assert!(super::is_ip_cidr_public_unicast("1.1.1.1".as_bytes()).is_err());
        assert!(super::is_ip_cidr_public_unicast("1.1.1/23".as_bytes()).is_err());
        assert!(super::is_ip_cidr_public_unicast("10.1.1.2/33".as_bytes()).is_err());
    }

    #[test]
    fn valid_public_ip() {
        // Technically unicast but yeah
        assert_eq!(super::is_ip_public_unicast("1.1.1.1".as_bytes()), Ok(true));
        assert_eq!(
            super::is_ip_public_unicast("10.10.100.254".as_bytes()),
            Ok(false)
        );
        assert_eq!(
            super::is_ip_public_unicast("10.10.100.254".as_bytes()),
            Ok(false)
        );
        assert_eq!(
            super::is_ip_public_unicast("172.10.100.254".as_bytes()),
            Ok(true)
        );
        assert_eq!(
            super::is_ip_public_unicast("172.20.100.254".as_bytes()),
            Ok(false)
        );
    }

    #[test]
    fn invalid_public_ip() {
        assert!(super::is_ip_public_unicast("1.1.1.1/23".as_bytes()).is_err());
        assert!(super::is_ip_public_unicast("1.1.1".as_bytes()).is_err());
        assert!(super::is_ip_public_unicast("1.1.1.1.1".as_bytes()).is_err());
    }

    #[test]
    fn valid_ip_in_cidr() {
        assert_eq!(
            super::is_part_of_cidr("34.0.0.1/24".as_bytes(), "34.0.0.254".as_bytes()),
            Ok(true),
        );
        assert_eq!(
            super::is_part_of_cidr("34.0.1.1/24".as_bytes(), "34.0.0.254".as_bytes()),
            Ok(false),
        );
    }

    #[test]
    fn invalid_ip_in_cidr() {
        assert!(super::is_part_of_cidr("34.0.0.1".as_bytes(), "34.0.0.254".as_bytes()).is_err());
    }
}