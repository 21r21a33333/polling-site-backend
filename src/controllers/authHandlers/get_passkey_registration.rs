use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AuthenticationResult, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

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
