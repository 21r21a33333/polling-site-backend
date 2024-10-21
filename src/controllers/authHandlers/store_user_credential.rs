use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AuthenticationResult, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

pub async fn store_user_credential(email: &str, passkey: &Passkey, pool: &sqlx::MySqlPool) {
    // Serialize the Passkey object to binary (you may need to use a specific serialization method)
    let passkey_blob = serde_json::to_string(passkey).expect("Failed to serialize Passkey");
    let cred_id = serde_json::to_string(&**passkey.cred_id()).expect("Failed to serialize Passkey");

    // Store the credential in the database
    sqlx::query(
        r#"
        INSERT INTO user_credentials (user_id, credential_id, passkey)
        VALUES ((SELECT id FROM users WHERE email = ? LIMIT 1), ?, ?)
        ON DUPLICATE KEY UPDATE
            credential_id = VALUES(credential_id),
            passkey = VALUES(passkey)
    "#,
    )
    .bind(email)
    .bind(cred_id) // Assuming credential ID is part of the Passkey
    .bind(passkey_blob)
    .execute(pool)
    .await
    .expect("Failed to store or update user credential");
    println!("User credential stored successfully for email: {}", email);
}
