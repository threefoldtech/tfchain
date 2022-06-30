pub fn valid_ipv6(input: &[u8]) -> bool {
    if input.len() < 3 {
        return false;
    }
    // Trap invalid input with leading single : so we don't have to track that in the loop.
    // Loop later verifies that the second char is actually a valid input char.
    if input[0] == b':' && input[1] != b':' {
        return false;
    }
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
                // Unconditionally increment section count. This will add 2 sections if :: is
                // detected, but that is fine since :: is only allowed if it masks at least 1 all 0
                // section.
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

    // Ending in a colon is not allowed. Also we need 8 sections or omitted sections.
    colons == 0 && (sections == 8 || double_colon_seen)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(super::valid_ipv6(b"::1"));
        assert!(!super::valid_ipv6(b"123"));
        assert!(!super::valid_ipv6(b":1"));
        assert!(super::valid_ipv6(b"200::1"));
        assert!(super::valid_ipv6(b"200:3412:023::22:1"));
        assert!(super::valid_ipv6(b"dead:babe:dead::babe:dead"));
        assert!(super::valid_ipv6(
            b"dead:babe:dead:babe:dead:babe:dead:babe"
        ));
        assert!(super::valid_ipv6(b"ffff:ff:ff::ac:0000:fffe"));
        assert!(!super::valid_ipv6(
            b"dead:babe:dead:babe:dead:babe:dead:babe:dead"
        ));
        assert!(!super::valid_ipv6(
            b"dead:babe:dead::babe:dead:babe:dead:babe"
        ));
        assert!(!super::valid_ipv6(b"fffg:ff:ff::ac:0000:fffe"));
        assert!(!super::valid_ipv6(b"ffff:::ac:0000:fffe"));
        assert!(!super::valid_ipv6(b"ffff::d::ac:0000:fffe"));
        assert!(!super::valid_ipv6(b"ffff::"));
        assert!(!super::valid_ipv6(b"ffff:"));
        assert!(!super::valid_ipv6(b"123"));
    }
}
