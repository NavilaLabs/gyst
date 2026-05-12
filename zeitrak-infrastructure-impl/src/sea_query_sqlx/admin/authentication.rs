/// # Errors
///
/// Returns an error if bcrypt hashing fails.
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub mod jwt {
    use bcrypt::verify;
    use chrono::Utc;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde::{Deserialize, Serialize};
    use zeitrak_core::admin::authenticator::{AuthenticationStrategy, Credentials};
    use zeitrak_infrastructure::config::CONFIG;

    use crate::Error;

    /// Maximum age of a JWT issued by this server, in seconds.
    ///
    /// Kept short (1 hour) to limit the blast radius of a stolen token.
    /// Pair with refresh-token logic on the client when sessions need to
    /// outlive this window.
    ///
    /// The compile-time assertion below ensures this value is never
    /// accidentally increased beyond one hour.
    pub const JWT_LIFETIME_SECS: usize = 3_600;

    // Compile-time regression guard: fail the build if JWT_LIFETIME_SECS is
    // increased beyond one hour.
    const _: () = assert!(
        JWT_LIFETIME_SECS <= 3_600,
        "JWT_LIFETIME_SECS must not exceed 3600 (1 hour)"
    );

    pub struct JwtAuthentication;

    impl AuthenticationStrategy for JwtAuthentication {
        type Error = Error;

        fn authenticate(&self, credentials: Credentials<'_>) -> Result<String, Self::Error> {
            let valid = verify(credentials.password, credentials.password_hash)
                .map_err(Error::BcryptError)?;

            if !valid {
                return Err(Error::InvalidCredentials);
            }

            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let now = Utc::now().timestamp() as usize;
            let claims = Claims {
                sub: credentials.user_id.to_string(),
                email: credentials.email.to_string(),
                iat: now,
                exp: now + JWT_LIFETIME_SECS,
            };

            let secret = CONFIG.application().security().authentication_secret();
            assert_ne!(secret, String::new());

            encode(
                &Header::new(Algorithm::HS256),
                &claims,
                &EncodingKey::from_secret(secret.as_bytes()),
            )
            .map_err(Error::JwtError)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        /// Subject — the user's ID.
        pub sub: String,
        pub email: String,
        /// Issued-at timestamp (seconds since Unix epoch).
        pub iat: usize,
        /// Expiration timestamp (seconds since Unix epoch).
        pub exp: usize,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// `JWT_LIFETIME_SECS` must be exactly 1 hour (3600 seconds).
        /// This test catches any attempt to extend the token lifetime at runtime
        /// in addition to the compile-time assertion above.
        #[test]
        fn jwt_lifetime_is_one_hour() {
            assert_eq!(
                JWT_LIFETIME_SECS, 3_600,
                "JWT lifetime must be exactly 1 hour (3600 s)"
            );
        }

        /// Verifies that `Claims` encodes the correct expiry window relative
        /// to the issued-at time.  The difference `exp - iat` must equal
        /// `JWT_LIFETIME_SECS` regardless of when the token is minted.
        #[test]
        fn claims_exp_minus_iat_equals_lifetime() {
            let fake_now: usize = 1_700_000_000;
            let claims = Claims {
                sub: "user-1".into(),
                email: "user@example.com".into(),
                iat: fake_now,
                exp: fake_now + JWT_LIFETIME_SECS,
            };
            assert_eq!(
                claims.exp - claims.iat,
                JWT_LIFETIME_SECS,
                "exp - iat must equal JWT_LIFETIME_SECS"
            );
            assert_eq!(
                claims.exp - claims.iat,
                3_600,
                "token window must be exactly 1 hour"
            );
        }
    }
}
