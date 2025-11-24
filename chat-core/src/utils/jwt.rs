use crate::models::user::CurUser;
use anyhow::Result;
use jwt_simple::{
    claims::Claims,
    prelude::{Duration, Ed25519KeyPair, Ed25519PublicKey, EdDSAKeyPairLike, EdDSAPublicKeyLike},
};

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISSUER: &str = "chat-server";
const JWT_AUD: &str = "chat-web";

pub struct EncodingKey(Ed25519KeyPair);

pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        let key_pair = Ed25519KeyPair::from_pem(pem)?;
        Ok(Self(key_pair))
    }

    pub fn sign(&self, user: impl Into<CurUser>) -> Result<String, jwt_simple::Error> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION))
            .with_issuer(JWT_ISSUER)
            .with_audience(JWT_AUD);

        self.0.sign(claims)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        let key = Ed25519PublicKey::from_pem(pem)?;
        Ok(Self(key))
    }

    pub fn verify(&self, token: &str) -> Result<CurUser, jwt_simple::Error> {
        let claims = self.0.verify_token::<CurUser>(token, None)?;
        Ok(claims.custom)
    }
}
/**
 * openssl genpkey -algorithm ED25519 -out ed25519-private.pem
 * openssl pkey -in ed25519-private.pem -pubout -out ed25519-public.pem
 */
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_jwt() {
        let key =
            EncodingKey::load(include_str!("../../fixtures/keys/ed25519-private.pem")).unwrap();
        let decoding_key =
            DecodingKey::load(include_str!("../../fixtures/keys/ed25519-public.pem")).unwrap();

        let user = CurUser {
            id: 1,
            ws_id: 1,
            fullname: "test".to_string(),
            email: "abc@gmail.com".to_string(),
        };
        let token = key.sign(user).unwrap();
        let user = decoding_key.verify(&token).unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.ws_id, 1);
        assert_eq!(user.fullname, "test");
        assert_eq!(user.email, "abc@gmail.com");
    }
}
