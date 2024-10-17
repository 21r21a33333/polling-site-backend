use std::{option, string};

use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
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

use webauthn_rs::prelude::*;

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

pub async fn update_credential_counter(email: &str, new_counter: u32, pool: &sqlx::MySqlPool) {
    // Update the stored credential counter in the database

    println!("Updating credential counter for email: {}", email);
}

// pub async fn get_passkey_registration(email: &str, pool: &sqlx::MySqlPool) -> PasskeyRegistration {
//     // Query the database to retrieve the stored passkey registration state
//     PasskeyRegistration::new() // Return a new PasskeyRegistration
// }

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

pub async fn get_passkey_registration(
    email: &str,
    pool: &sqlx::MySqlPool,
) -> Option<PasskeyRegistration> {
    // Query the database to retrieve the stored passkey registration state
    let query = r#"
        SELECT registration_data
        FROM user_passkey_registrations
        WHERE user_id = (SELECT id FROM users WHERE email = ?)
    "#;

    let row = sqlx::query(&query).bind(email).fetch_one(pool).await.ok()?;

    let registration_data: String = row.get("registration_data");
    from_str::<PasskeyRegistration>(&registration_data).ok()
}

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

pub async fn get_user_credentials(
    email: &str,
    pool: &sqlx::MySqlPool,
) -> Option<Vec<CredentialID>> {
    let query = r#"
        SELECT user_credentials.credential_id 
        FROM user_credentials
        INNER JOIN users ON user_credentials.user_id = users.id
        WHERE users.email = ?
    "#;

    let rows = sqlx::query(query).bind(email).fetch_all(pool).await.ok()?;

    let credentials = rows
        .into_iter()
        .map(|row| row.get::<Vec<u8>, _>("credential_id").into())
        .collect::<Vec<CredentialID>>();

    Some(credentials)
}
