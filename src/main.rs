use std::fs;
use std::path::Path;
use aes_gcm::aead::{KeyInit, Aead};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .unwrap();
    
    key
}

fn encrypt(password: &str, plaintext: &[u8]) -> Vec<u8> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    
    getrandom::fill(&mut salt).unwrap();
    getrandom::fill(&mut nonce).unwrap();

    let n = Nonce::try_from(&nonce[..]).unwrap();

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();

    let ct = cipher.encrypt(&n, plaintext).unwrap();

    [&salt[..], &nonce[..], &ct[..]].concat()

}

fn main() {
    println!("Hello, world!");

    let path = Path::new("diary.enc");

    if !path.exists() { 
        let password = rpassword::prompt_password("Enter new password: ").unwrap();
        let repeat = rpassword::prompt_password("Re-enter password: ").unwrap();

        assert_eq!(password, repeat, "Passwords do not match.");

        fs::write(path, encrypt(&password, b""))
            .expect("Failed to create diary");

        println!("Diary created");
    }
    
    let data = fs::read(path);
    println!("{:?}", data);
}
