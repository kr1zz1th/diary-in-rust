use std::fs;
use std::path::Path;
use aes_gcm::aead::{Aead, Generate, KeyInit};
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
    getrandom::fill(&mut salt).unwrap();

    let n = Nonce::generate();

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();

    let ct = cipher.encrypt(&n, plaintext).unwrap();

    [&salt[..], &n, &ct[..]].concat()

}

fn decrypt(password: &str, encrypted_stuff: &[u8]) -> Vec<u8> {
    let n = Nonce::try_from(&encrypted_stuff[16..28]).unwrap();
    let salt = &encrypted_stuff[..16];

    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    
    let ct = cipher.decrypt(&n, &encrypted_stuff[28..]).expect("wrong password");

    ct
}

fn main() {
    println!("Hello, world!");

    let path = Path::new("diary.enc");

    if !path.exists() { 
        let npassword = rpassword::prompt_password("Enter new password: ").unwrap();
        let repeat = rpassword::prompt_password("Re-enter password: ").unwrap();

        assert_eq!(npassword, repeat, "Passwords do not match.");

        fs::write(path, encrypt(&npassword, b""))
            .expect("Failed to create diary");

        println!("Diary created");
    }
    
    let data = fs::read(path).unwrap();

    let password = rpassword::prompt_password("Enter password: ").unwrap();
    let decrypted_data = decrypt(&password, &data);

    println!("{:?}", decrypted_data);
}
