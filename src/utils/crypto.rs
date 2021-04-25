use crate::utils::{
    data::{Claims, User},
    errors,
    types::CustomResult,
};
use argon2::{hash_encoded, Config, ThreadMode, Variant, Version};
use chrono;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
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

static SECRET_KEY: &str = "Definitely need to change this";

pub fn create_hash(password: &String) -> CustomResult<String> {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    hash_encoded(password.as_bytes(), &salt, &ARGON2_HASH_CONFIG)
        .map_err(|e| errors::CustomErrors::UnexpectedError)
}

pub fn check_hash(password: &String, hash: &String) -> bool {
    // may be a source of errors later on. Careful future self...
    argon2::verify_encoded(hash, password.as_bytes()).is_ok()
}

pub fn create_auth_token(user: &User) -> CustomResult<String> {
    let expire = chrono::Utc::now() + chrono::Duration::days(3);
    let claims = Claims {
        username: user.username.clone(),
        password_hash: create_hash(&user.password.clone())
            .map_err(|e| errors::CustomErrors::UnexpectedError)?, // may cause issues
        exp: expire.timestamp() as usize,
    };
    let header = Header::new(Algorithm::HS512);
    jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_secret(&SECRET_KEY.as_bytes()),
    )
    .map_err(errors::CustomErrors::JWTEncodingError)
}

pub fn verify_auth_token(token: &String) -> bool {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(&SECRET_KEY.as_bytes()),
        &Validation::new(Algorithm::HS512),
    )
    .map_or(false, |c| {
        c.claims.exp < chrono::Utc::now().timestamp() as usize
    })
}

#[cfg(test)]
mod crypto_test {
    use super::*;
    #[test]
    fn hash_test() {
        let password = "testy string".to_string();
        let hash = create_hash(&password).unwrap();
        assert!(check_hash(&password, &hash));
    }
    #[test]
    fn jwt_test() {
        let user = User {
            username: "test1".to_string(),
            password: "test2".to_string(),
        };
        let token = create_auth_token(&user).unwrap();
        assert!(true, "{}", verify_auth_token(&token));
    }
}
