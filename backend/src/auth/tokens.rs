use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn random_token(bytes: usize) -> String {
    let mut value = vec![0u8; bytes];
    OsRng.fill_bytes(&mut value);
    URL_SAFE_NO_PAD.encode(value)
}

pub fn access_token() -> String {
    random_token(32)
}

pub fn refresh_token() -> String {
    random_token(64)
}

pub fn hash_token(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex_encode(&digest)
}

pub fn random_otp() -> String {
    let mut bytes = [0u8; 4];
    OsRng.fill_bytes(&mut bytes);
    format!("{:06}", u32::from_le_bytes(bytes) % 1_000_000)
}

pub fn random_family_id() -> Uuid {
    Uuid::new_v4()
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_have_expected_entropy_lengths() {
        assert_eq!(URL_SAFE_NO_PAD.decode(access_token()).unwrap().len(), 32);
        assert_eq!(URL_SAFE_NO_PAD.decode(refresh_token()).unwrap().len(), 64);
    }

    #[test]
    fn hashing_is_deterministic_and_not_plaintext() {
        let value = "test-token";
        let hash = hash_token(value);
        assert_eq!(hash, hash_token(value));
        assert_ne!(hash, value);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn otp_is_six_digits() {
        let otp = random_otp();
        assert_eq!(otp.len(), 6);
        assert!(otp.bytes().all(|b| b.is_ascii_digit()));
    }
}
