use soroban_sdk::{xdr::ToXdr, xdr::FromXdr, Bytes, Env, String};

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

pub fn get_address_from(network_address: &String, env: &Env) -> String {
    let mut nid = Bytes::new(&env);
    let mut account = Bytes::new(&env);

    let addr_slice = get_bytes_from_string(&env,network_address.clone());

    let mut has_seperator = false;
    for (index, value) in addr_slice.clone().iter().enumerate() {
        if has_seperator {
            account.append(&addr_slice.slice(index as u32..addr_slice.len()));
            break;
        } else if value == 47 {
            has_seperator = true;
        } else {
            nid.push_back(value)
        }
    }

    if !has_seperator {
        panic!("Invalid network address")
    }
    

    get_string_from_bytes(&env, account)
    
}

pub fn get_bytes_from_string(env: &Env, value: String) -> Bytes {
    let bytes = value.to_xdr(&env);

    if bytes.get(6).unwrap() > 0 {
        panic!("Invalid network address length")
    }

    let value_len = bytes.get(7).unwrap();
    let slice = bytes.slice(8..value_len as u32 + 8);
    slice
}

pub fn get_string_from_bytes(e: &Env, bytes: Bytes) -> String {
    let mut bytes_xdr = bytes.to_xdr(&e);
    bytes_xdr.set(3, 14);

    String::from_xdr(&e, &bytes_xdr).unwrap()
}