extern crate openssl;

use openssl::hash::MessageDigest;
use openssl::symm::Cipher;

// TODO: Remove all unwrap's

const SALT_SIZE: usize = 32;
const PBKDF_ITERS: usize = 10000;
// const HASH: MessageDigest = MessageDigest::sha256();
// const CIPHER: Cipher = Cipher::aes_256_gcm();
const KEY_SIZE: usize = 256 / 8;
const IV_SIZE: usize = KEY_SIZE;

pub struct HiddenData {
    salt: Vec<u8>,
    blocks: Vec<EncryptedBlock>,
    hidden_data: Vec<u8>, // TODO: remove this (that was put only to make tests pass)
}

struct EncryptedBlock {
    iv: [u8; IV_SIZE],
    data: Vec<u8>,
}

/**
 * Encrypts data with key returns a block that can be decrypted with the key
 *
 * data and key will be erased at the end of the function
 */
fn encrypt_and_destroy_key(mut data: Vec<u8>, mut key: Vec<u8>) -> EncryptedBlock {
    let mut iv = [0; IV_SIZE];
    openssl::rand::rand_bytes(&mut iv).unwrap();

    // Encrypt
    let enc = openssl::symm::encrypt(/* CIPHER */ Cipher::aes_256_gcm(), &key, Some(&iv), &data);

    // Clean up
    for x in data.iter_mut() {
        *x = 0;
    }
    for x in key.iter_mut() {
        *x = 0;
    }

    // Return
    EncryptedBlock { iv: iv, data: enc.unwrap() }
}

/**
 * Encrypts data and returns both the key used and a block that can be decrypted with the key
 *
 * data will be erased at the end of the function
 */
fn encrypt_and_destroy(data: Vec<u8>) -> (Vec<u8>, EncryptedBlock) {
    // Generate the parameters
    let mut key = vec![0; KEY_SIZE];
    openssl::rand::rand_bytes(&mut key).unwrap();
    let keyret = key.clone();

    // Return
    (keyret, encrypt_and_destroy_key(data, key))
}

/**
 * Derives a key from a secret, and returns a couple (salt, key)
 *
 * The secret will be erased at the end of the function
 */
fn derive_key(secret: &mut [u8]) -> (Vec<u8>, Vec<u8>) {
    let mut salt = vec![0; SALT_SIZE];
    openssl::rand::rand_bytes(&mut salt).unwrap();

    // Generate key
    let mut key = vec![0; KEY_SIZE];
    openssl::pkcs5::pbkdf2_hmac(&secret, &salt, PBKDF_ITERS, /* HASH */ MessageDigest::sha256(), &mut key).unwrap();

    // Clean up
    for x in secret.iter_mut() {
        *x = 0;
    }

    // Return
    (salt, key)
}

/**
 * Hides data so that it cannot be recovered by a cold boot attack without the secret secret.
 *
 * Please note both data and secret will be erased at the end of this function, so that it is hard
 * to forget cleaning them up.
 */
pub fn hide(mut data: Vec<u8>, secret: &mut [u8]) -> HiddenData {
    let mut blocks = Vec::new();

    // TODO: remove
    let hidden_data = data.clone();

    // Encrypt data with random key
    let (key, block) = encrypt_and_destroy(data);
    blocks.push(block);

    // Encrypt key with random key a number of times
    let mut oldkey = key;
    for _ in 0..1000 { // TODO: make this parameterized
        let (key, block) = encrypt_and_destroy(oldkey);
        blocks.push(block);
        oldkey = key;
    }

    // Encrypt last random key with the secret
    let (salt, key) = derive_key(secret);
    blocks.push(encrypt_and_destroy_key(oldkey, key));

    // Return
    HiddenData {
        salt: salt,
        blocks: blocks,
        hidden_data: hidden_data,
    }
}

/**
 * Recovers data hidden with secret secret.
 *
 * Please note secret will be erased at the end of this function, so that it is hard to forget
 * cleaning it up.
 */
pub fn recover(data: HiddenData, secret: &mut [u8]) -> Vec<u8> {
    // TODO
    // Erase secret
    // TODO
    // Return
    data.hidden_data
}


#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn it_works() {
        let mut secret1 = [0, 1, 2, 3];
        let mut secret2 = secret1.clone();
        let data1 = vec![4, 5, 6, 6];
        let data2 = data1.clone();
        assert_eq!(*recover(hide(data1, &mut secret1), &mut secret2), *data2);
    }
}
