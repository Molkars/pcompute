use anyhow::{anyhow, Context};
use argon2::Argon2;
use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use password_hash::rand_core::OsRng;

pub fn hash_password(password: impl AsRef<[u8]>) -> anyhow::Result<String> {
    let pepper = pepper_password(password)?;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(pepper.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to hash password: {}", e))
        .map(|hash| hash.to_string())
}

pub fn verify_password(password: impl AsRef<[u8]>, hash: impl AsRef<str>) -> anyhow::Result<bool> {
    let hash = PasswordHash::new(hash.as_ref())
        .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
    let peppered_password = pepper_password(password)?;

    let argon2 = Argon2::default();
    match argon2.verify_password(peppered_password.as_bytes(), &hash) {
        Ok(()) => Ok(true),
        Err(e) if matches!(e, password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow!("Failed to verify password: {}", e)),
    }
}

fn pepper_password(password: impl AsRef<[u8]>) -> anyhow::Result<String> {
    let password = password.as_ref();
    let pepper = pepper()
        .context("Failed to get password pepper")?;
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_ref(), &pepper)
        .map_err(|e| anyhow!("Failed to hash password: {}", e))
        .map(|hash| hash.to_string())
}

fn pepper() -> anyhow::Result<SaltString> {
    let pepper = std::env::var("PASSOWRD_PEPPER")
        .context("PASSWORD_PEPPER must be set & must be 48 bytes long")?;

    SaltString::encode_b64(pepper.as_bytes())
        .map_err(|e| anyhow!("Failed to encode password pepper: {}", e))
}