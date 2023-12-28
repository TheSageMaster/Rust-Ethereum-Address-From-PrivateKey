use ethnum::U256;
use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};
use hex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[macro_use]
extern crate arrayref;

fn main() -> io::Result<()> {
    let private_keys_hex = read_private_keys_from_file("private_keys.txt")?;

    for key_hex in private_keys_hex {
        let key_bytes = hex::decode(key_hex.trim()).expect("Invalid hex format");
        let private_key = U256::from_be_bytes(*array_ref!(key_bytes, 0, 32));
        let eip55_address = generate_ethereum_address(private_key);
        
        println!("EIP-55 Ethereum Address: 0x{}", eip55_address);
    }

    Ok(())
}

fn generate_ethereum_address(private_key: U256) -> String {
    let private_key_bytes = u256_to_bytes_be(private_key);
    let signing_key = SigningKey::from_bytes(&private_key_bytes.into()).expect("Invalid private key");
    let public_key = signing_key.verifying_key().to_encoded_point(false);

    let hash = Keccak256::digest(&public_key.as_bytes()[1..]);

    // Generate EIP-55 compliant address
    let address = hex::encode(&hash[12..]); // Ethereum address is last 20 bytes
    apply_eip55_checksum(&address)
}

fn apply_eip55_checksum(address: &str) -> String {
    let hash = Keccak256::digest(address.as_bytes());
    let hash_hex = hex::encode(hash);

    address
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_ascii_hexdigit() && c.is_ascii_alphabetic() {
                let hash_char = hash_hex.chars().nth(i).unwrap_or('0');
                if hash_char > '7' {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                }
            } else {
                c
            }
        })
        .collect()
}

fn read_private_keys_from_file<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let buf_reader = io::BufReader::new(file);
    buf_reader.lines().collect()
}

// Convert U256 to a 32-byte array in big-endian format
fn u256_to_bytes_be(value: U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let u128_parts = value.0; // Extract the [u128; 2] array

    // Process each u128 part
    for (i, &part) in u128_parts.iter().enumerate() {
        // Convert each u128 to big-endian bytes
        let part_bytes = part.to_be_bytes();

        // Determine the starting index for copying bytes into the final array
        let start_index = i * 16; // Each u128 has 16 bytes

        // Copy bytes into the appropriate position of the final array
        bytes[start_index..start_index + 16].copy_from_slice(&part_bytes);
    }

    bytes
}