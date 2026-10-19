use std::fs;
use std::path::Path;
use aes_gcm::aead::{Aead, Generate, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use std::os::unix::fs::OpenOptionsExt;
use std::io::Write;
use std::process::Command;

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
    
    cipher.decrypt(&n, &encrypted_stuff[28..]).expect("wrong password")
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

    // Write into /tmp

    let mut tmp_num = [0u8; 8];
    getrandom::fill(&mut tmp_num).unwrap();
    let tmp_num1 = u64::from_le_bytes(tmp_num);
    
    let tmp_string = format!("/tmp/diary-{:016x}.txt", tmp_num1);
    let tmp_path = Path::new(&tmp_string);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp_path)
        .unwrap();

    file.write_all(&decrypted_data).unwrap();
    drop(file);

    let status = Command::new("nvim")
        .args(["-n", "-i", "NONE", "-c", "set noundofile nobackup nowritebackup"])
        .arg(tmp_path)
        .status()
        .unwrap();


    let edited = fs::read(tmp_path).unwrap();
    fs::write(tmp_path, vec![0u8; edited.len()]).unwrap();
    fs::remove_file(tmp_path).unwrap();

    if !status.success() {
        println!("nvim crashed, all changes are discarded");
        return;
    }

    fs::write("diary.enc", encrypt(&password, &edited)).unwrap();

    println!("Saved diary");
}
