use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AuthenticationResult, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

pub async fn get_passkey_auth_state(email: &str, pool: &sqlx::MySqlPool) -> PasskeyAuthentication {
    // Query the database to retrieve the stored passkey authentication state
    let query = r#"
            SELECT state
            FROM passkey_auth_state
            WHERE user_id = (SELECT id FROM users WHERE email = ?)
        "#;

    let row = sqlx::query(query)
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("Failed to fetch passkey authentication state");

    let auth_state: String = row.get("state");
    let passkeyAuth = from_str(&auth_state).expect("Failed to deserialize PasskeyAuthentication");

    passkeyAuth
}
