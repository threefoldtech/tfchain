pub fn valid_ipv6(input: &[u8]) -> bool {
    let mut current_group_elements = 0;
    let mut sections = 1;
    let mut double_colon_seen = false;
    let mut colons = 0;
    for b in input {
        match b {
            b'a'..=b'f' | b'0'..=b'9' => {
                current_group_elements += 1;
                colons = 0;
            }
            b':' => {
                colons += 1;
                sections += 1;
                current_group_elements = 0;
            }
            _ => return false,
        };
        if current_group_elements > 4
            || sections > 8
            || colons > 2
            || (colons > 1 && double_colon_seen)
        {
            return false;
        }
        if colons == 2 {
            double_colon_seen = true;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::valid_ipv6;

    #[test]
    fn it_works() {
        assert!(valid_ipv6(b"::1"));
        assert!(valid_ipv6(b"200::1"));
        assert!(valid_ipv6(b"200:3412:023::22:1"));
        assert!(valid_ipv6(b"dead:babe:dead::babe:dead"));
        assert!(valid_ipv6(b"dead:babe:dead:babe:dead:babe:dead:babe"));
        assert!(valid_ipv6(b"ffff:ff:ff::ac:0000:fffe"));
        assert!(!valid_ipv6(b"fffg:ff:ff::ac:0000:fffe"));
        assert!(!valid_ipv6(b"fffg:::ac:0000:fffe"));
        assert!(!valid_ipv6(b"fffg::d::ac:0000:fffe"));
    }
}