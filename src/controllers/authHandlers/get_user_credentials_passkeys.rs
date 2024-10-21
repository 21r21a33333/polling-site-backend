use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AuthenticationResult, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

pub async fn get_user_credentials_passkeys(
    email: &str,
    pool: &sqlx::MySqlPool,
) -> Option<Vec<Passkey>> {
    // Query to get the serialized passkeys from the database
    let query = r#"
        SELECT uc.passkey
        FROM user_credentials uc
        INNER JOIN users u ON uc.user_id = u.id
        WHERE u.email = ?
    "#;

    let rows = sqlx::query(query)
        .bind(email) // Bind the email to the query
        .fetch_all(pool)
        .await
        .ok()?; // If fetching fails, return None

    if rows.is_empty() {
        return None; // No passkeys found for the user
    }

    let passkeys = rows
        .into_iter()
        .map(|row| {
            let passkey: String = row.get("passkey");
            serde_json::from_str(&passkey).expect("Failed to deserialize Passkey")
        })
        .collect::<Vec<Passkey>>();

    Some(passkeys)
}
