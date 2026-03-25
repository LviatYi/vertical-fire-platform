use base64::Engine;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

const RSA_PREFIX: &str = "rsa:";
const BASE64_PREFIX: &str = "base64:";

fn configured_private_key_pem() -> Option<&'static str> {
    let pem = crate::default_config::runtime()
        .jenkins_pwd_rsa_private_key_pem
        .trim();
    (!pem.is_empty()).then_some(pem)
}

pub fn encrypt_jenkins_pwd(plain: &str) -> Result<String, String> {
    encrypt_jenkins_pwd_with_private_key_pem(configured_private_key_pem(), plain)
}

pub fn decrypt_jenkins_pwd(cipher: &str) -> Option<String> {
    decrypt_jenkins_pwd_with_private_key_pem(configured_private_key_pem(), cipher)
}

fn encrypt_jenkins_pwd_with_private_key_pem(
    private_key_pem: Option<&str>,
    plain: &str,
) -> Result<String, String> {
    match private_key_pem {
        Some(private_key_pem) => {
            let private_key = parse_private_key(private_key_pem)?;
            let public_key = RsaPublicKey::from(&private_key);
            let mut rng = rsa::rand_core::OsRng;
            let encrypted = public_key
                .encrypt(&mut rng, Oaep::new::<Sha256>(), plain.as_bytes())
                .map_err(|err| err.to_string())?;

            Ok(format!(
                "{RSA_PREFIX}{}",
                base64::prelude::BASE64_STANDARD.encode(encrypted)
            ))
        }
        None => Ok(format!(
            "{BASE64_PREFIX}{}",
            base64::prelude::BASE64_STANDARD.encode(plain)
        )),
    }
}

fn decrypt_jenkins_pwd_with_private_key_pem(
    private_key_pem: Option<&str>,
    cipher: &str,
) -> Option<String> {
    if let Some(cipher) = cipher.strip_prefix(RSA_PREFIX) {
        let private_key = parse_private_key(private_key_pem?).ok()?;
        let decoded = base64::prelude::BASE64_STANDARD.decode(cipher).ok()?;
        let decrypted = private_key.decrypt(Oaep::new::<Sha256>(), &decoded).ok()?;
        return String::from_utf8(decrypted).ok();
    }

    let cipher = cipher.strip_prefix(BASE64_PREFIX).unwrap_or(cipher);
    base64::prelude::BASE64_STANDARD
        .decode(cipher)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

fn parse_private_key(private_key_pem: &str) -> Result<RsaPrivateKey, String> {
    RsaPrivateKey::from_pkcs8_pem(private_key_pem).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs8::{EncodePrivateKey, LineEnding};

    fn create_test_private_key_pem() -> String {
        let mut rng = rsa::rand_core::OsRng;
        RsaPrivateKey::new(&mut rng, 2048)
            .unwrap()
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap()
            .to_string()
    }

    #[test]
    fn test_encrypt_and_decrypt_jenkins_pwd_with_rsa_key() {
        let private_key_pem = create_test_private_key_pem();
        let plain = "my-password";

        let encrypted =
            encrypt_jenkins_pwd_with_private_key_pem(Some(private_key_pem.as_str()), plain).unwrap();

        assert!(encrypted.starts_with(RSA_PREFIX));
        assert_eq!(
            decrypt_jenkins_pwd_with_private_key_pem(Some(private_key_pem.as_str()), &encrypted),
            Some(plain.to_string())
        );
    }

    #[test]
    fn test_encrypt_and_decrypt_jenkins_pwd_with_fallback_base64() {
        let plain = "legacy-password";

        let encrypted = encrypt_jenkins_pwd_with_private_key_pem(None, plain).unwrap();

        assert!(encrypted.starts_with(BASE64_PREFIX));
        assert_eq!(
            decrypt_jenkins_pwd_with_private_key_pem(None, &encrypted),
            Some(plain.to_string())
        );
    }

    #[test]
    fn test_decrypt_legacy_base64_without_prefix() {
        let plain = "legacy-password";
        let encrypted = base64::prelude::BASE64_STANDARD.encode(plain);

        assert_eq!(
            decrypt_jenkins_pwd_with_private_key_pem(None, &encrypted),
            Some(plain.to_string())
        );
    }
}
