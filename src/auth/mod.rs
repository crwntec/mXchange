use crate::db::get_user_by_address;
use md5::Md5;
use hmac::{Hmac, Mac};
use hex;

type HmacMD5 = Hmac<Md5>;

pub fn validate_auth(address: String, password_hash: String, challenge: String) -> bool {
    match get_user_by_address(address) {
        Ok(user) => {
            let mut mac = HmacMD5::new_from_slice(user.password.as_bytes()).expect("Error while processing hash");
            mac.update(challenge.as_bytes());
            let result = mac.finalize();
            let computed_hash = hex::encode(result.into_bytes());
            password_hash == computed_hash
        }
        Err(e) => {
            println!("Error while validating auth: {}", e);
            false
        }
    }
}