use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AuthenticationResult, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

// Insert passkey authentication state into the database
pub async fn store_passkey_auth_state(
    email: &str,
    auth_result: &PasskeyAuthentication,
    pool: &sqlx::MySqlPool,
) {
    let auth_state = to_string(auth_result).expect("Failed to serialize PasskeyAuthentication");

    let query = r#"
            INSERT INTO passkey_auth_state (user_id, state)
            VALUES (
                (SELECT id FROM users WHERE email = ?),
                ?
            )
            ON DUPLICATE KEY UPDATE
                state = VALUES(state)
        "#;

    let result = sqlx::query(query)
        .bind(email)
        .bind(auth_state)
        .execute(pool)
        .await;

    match result {
        Ok(_) => {
            println!(
                "Passkey authentication state stored successfully for email: {}",
                email
            );
        }
        Err(err) => {
            eprintln!("Failed to store passkey authentication state: {}", err);
        }
    }
}
