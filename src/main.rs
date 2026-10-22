// i didnt use ai for these comments, im just explainign the code here instead of doing it in the
// report
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
    //Derive the key
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .unwrap();
    
    key
}

fn encrypt(password: &str, plaintext: &[u8]) -> Vec<u8> {
    // Create a random salt
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt).unwrap();

    // Create a random nonce
    let n = Nonce::generate();

    //Derive the key using the salt and the password
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();

    let ct = cipher.encrypt(&n, plaintext).unwrap();

    // Return the salt, nonce, and ciphertext
    [&salt[..], &n, &ct[..]].concat()

}

fn decrypt(password: &str, encrypted_stuff: &[u8]) -> Vec<u8> {
    // Get the nonce nad the salt
    let n = Nonce::try_from(&encrypted_stuff[16..28]).unwrap();
    let salt = &encrypted_stuff[..16];

    // Derive key
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    
    //Decrypt, and if it fails, return "Wrong password"
    cipher.decrypt(&n, &encrypted_stuff[28..]).expect("wrong password")
}

fn main() {
    // Get the path of the encrypted file
    let path = Path::new("diary.enc");

    // If the diary doesnt exist
    if !path.exists() { 
        let npassword = rpassword::prompt_password("Enter new password: ").unwrap();
        let repeat = rpassword::prompt_password("Re-enter password: ").unwrap();

        // chekc if passwords match
        assert_eq!(npassword, repeat, "Passwords do not match.");

        fs::write(path, encrypt(&npassword, b""))
            .expect("Failed to create diary");

        println!("Diary created");
    }
    
    let data = fs::read(path).unwrap();

    let password = rpassword::prompt_password("Enter password: ").unwrap();
    let decrypted_data = decrypt(&password, &data);

    // Make a random number
    let mut tmp_num = [0u8; 8];
    getrandom::fill(&mut tmp_num).unwrap();
    let tmp_num1 = u64::from_le_bytes(tmp_num);
    
    //make the random path
    let tmp_string = format!("/tmp/diary-{:016x}.txt", tmp_num1);
    let tmp_path = Path::new(&tmp_string);


    // Open the file in 0o600 mode, so only the owner (us) can rw into it
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp_path)
        .unwrap();

    //Write the decrypted data and remove it from memory
    file.write_all(&decrypted_data).unwrap();
    drop(file);

    //open nvim with options such that it doesn't store the content somewhere in its cache
    let status = Command::new("nvim")
        .args(["-n", "-i", "NONE", "-c", "set noundofile nobackup nowritebackup"])
        .arg(tmp_path)
        .status()
        .unwrap();

    // Now when you quit nvim, it'll continue executing from here
    // Read the file
    let edited = fs::read(tmp_path).unwrap();
    // Fill the file wiht blank values
    fs::write(tmp_path, vec![0u8; edited.len()]).unwrap();
    //Delte the file
    fs::remove_file(tmp_path).unwrap();

    if !status.success() {
        println!("nvim crashed, all changes are discarded");
        return;
    }
    
    //Write the encrypted stuff into the file
    fs::write("diary.enc", encrypt(&password, &edited)).unwrap();

    println!("Saved diary");
}
