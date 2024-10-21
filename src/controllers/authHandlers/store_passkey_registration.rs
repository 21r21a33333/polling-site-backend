use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AuthenticationResult, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

pub async fn store_passkey_registration(
    email: &str,
    display_name: &str,
    registration: &PasskeyRegistration,
    pool: &MySqlPool,
) {
    let query = r#"
        INSERT INTO users (email, display_name)
        VALUES (?, ?)
    "#;
    let result = sqlx::query(query)
        .bind(email)
        .bind(display_name)
        .execute(pool)
        .await;
    match result {
        Ok(_) => {
            println!("User stored successfully for email: {}", email);
        }
        Err(err) => {
            eprintln!("Failed to store user: {}", err);
        }
    }

    // Serialize the registration object to JSON
    let registration_data = to_string(registration).unwrap();

    // SQL query to insert passkey registration state into the database
    let query = r#"
        INSERT INTO user_passkey_registrations (user_id, registration_data)
        VALUES (
            (SELECT id FROM users WHERE email = ?),
            ?
        )
    "#;

    // Execute the insert query
    let result = sqlx::query(query)
        .bind(email) // Bind the email to get user ID
        .bind(registration_data) // Bind the serialized registration data
        .execute(pool) // Execute the query on the provided MySQL pool
        .await;

    match result {
        Ok(_) => {
            println!(
                "Passkey registration stored successfully for email: {}",
                email
            );
        }
        Err(err) => {
            eprintln!("Failed to store passkey registration: {}", err);
        }
    }
}
