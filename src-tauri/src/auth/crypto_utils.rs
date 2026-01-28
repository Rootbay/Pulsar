use crate::auth::types::{
    Argon2ParamsConfig, ARGON2_MAX_MEMORY_KIB, ARGON2_MAX_PARALLELISM, ARGON2_MAX_TIME_COST,
    ARGON2_MIN_MEMORY_KIB,
};
use crate::error::{Error, Result};
use argon2::{Algorithm, Argon2, Version};
use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive_key(password: &str, salt: &[u8], params: &Argon2ParamsConfig) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    let params = params.to_params()?;
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| Error::Internal(format!("Failed to derive key: {}", e)))?;
    Ok(key)
}

pub fn derive_metadata_mac_key(master_key: &[u8]) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut out = [0u8; 32];
    hk.expand(b"pulsar:meta", &mut out)
        .map_err(|_| Error::Internal("Failed to derive metadata MAC key".to_string()))?;
    Ok(out)
}

pub fn validate_argon_params(params: &Argon2ParamsConfig) -> Result<()> {
    if params.memory_kib < ARGON2_MIN_MEMORY_KIB {
        return Err(Error::Validation(
            "Argon2 memory must be at least 8 MiB.".to_string(),
        ));
    }
    if params.memory_kib > ARGON2_MAX_MEMORY_KIB {
        return Err(Error::Validation("Argon2 memory is too high.".to_string()));
    }

    if params.time_cost == 0 {
        return Err(Error::Validation(
            "Argon2 time cost must be at least 1.".to_string(),
        ));
    }
    if params.time_cost > ARGON2_MAX_TIME_COST {
        return Err(Error::Validation(
            "Argon2 time cost is too high.".to_string(),
        ));
    }

    if params.parallelism == 0 {
        return Err(Error::Validation(
            "Argon2 parallelism must be at least 1.".to_string(),
        ));
    }
    if params.parallelism > ARGON2_MAX_PARALLELISM {
        return Err(Error::Validation(
            "Argon2 parallelism is too high.".to_string(),
        ));
    }

    Ok(())
}
