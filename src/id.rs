use std::env;

use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

pub fn generate_file_id() -> String {
    let id_length = env::var("ID_LENGTH")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .expect("Error while parsing environment variable 'ID_LENGTH'");
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(id_length)
        .map(char::from)
        .collect()
}

pub fn generate_change_key() -> String {
    let change_key_length = env::var("CHANGE_KEY_LENGTH")
        .unwrap_or_else(|_| "64".to_string())
        .parse::<usize>()
        .expect("Error while parsing enviroment variable 'CHANGE_KEY_LENGTH'");
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(change_key_length)
        .map(char::from)
        .collect()
}

pub fn hash_change_key(key: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.finalize()[..].to_vec()
}
