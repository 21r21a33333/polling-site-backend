use actix_web::web::Json;
use serde_json::{from_str, to_string};
use sqlx::{query, MySqlPool, Row};
use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AuthenticationResult, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

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
