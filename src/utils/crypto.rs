use crate::utils::errors;
use crate::utils::types::CustomResult;
use argon2;
use argon2::{hash_encoded, Config, ThreadMode, Variant, Version};
use rand::Rng;

static ARGON2_HASH_CONFIG: Config = Config {
    variant: Variant::Argon2i,
    version: Version::Version13,
    mem_cost: 65536,
    time_cost: 10,
    lanes: 4,
    thread_mode: ThreadMode::Parallel,
    secret: &[],
    ad: &[],
    hash_length: 32,
};

pub fn create_hash(password: &String) -> CustomResult<String> {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    hash_encoded(password.as_bytes(), &salt, &ARGON2_HASH_CONFIG)
        .map_err(|e| errors::CustomErrors::UnexpectedError)
}
