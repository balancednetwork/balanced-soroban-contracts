use soroban_sdk::{String, Bytes};

pub fn is_valid_string_address(address: &String) -> bool {
    if address.len() != 56 {
        return false;
    }

    let mut address_bytes = [0u8; 56];
    address.copy_into_slice(&mut address_bytes);

    let mut is_valid = true;

    if address_bytes[0] != b'G' && address_bytes[0] != b'C' {
        is_valid = false;
    }

    for &byte in &address_bytes {
        if !is_valid_base32(byte) {
            is_valid = false;
            break;
        }
    }

    is_valid
}

pub fn is_valid_bytes_address(address: &Bytes) -> bool {
    if address.len() != 56 {
        return false;
    }
    if address.get(0).unwrap() != b'G' && address.get(0).unwrap() != b'C'  {
        return false;
    }

    for i in 0..56 {
        let byte = address.get(i).unwrap();
        if !is_valid_base32(byte) {
            return false;
        }
    }

    true
}



fn is_valid_base32(byte: u8) -> bool {
    match byte {
        b'A'..=b'Z' | b'2'..=b'7' => true,
        _ => false,
    }
}