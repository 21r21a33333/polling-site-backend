use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Responder,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;
use webauthn_rs::prelude::{
    Base64UrlSafeData, CreationChallengeResponse, CredentialID, Passkey, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use crate::{
    config::create_webauthn_instance, get_passkey_auth_state, get_passkey_registration,
    get_user_credentials, get_user_credentials_passkeys, store_passkey_auth_state,
    store_passkey_registration, store_user_credential, update_credential_counter,
};

#[derive(Deserialize)]
struct StartRegistrationRequest {
    email: String,
    display_name: String,
}

#[derive(Serialize)]
struct RegisterStartResponse {
    challenge: Base64UrlSafeData,
    user_id: Uuid,
}

#[post("/register/start")]
async fn register_start(
    pool: Data<MySqlPool>,
    body: web::Json<StartRegistrationRequest>,
) -> impl Responder {
    println!("/POST register/start");
    let data = create_webauthn_instance();
    let user_unique_id = Uuid::new_v4(); // Generate a new UUID for the user
    let email = &body.email;
    let display_name = &body.display_name;

    // Check if the user already exists with the given email
    let user_exists = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE email = ?", email)
        .fetch_one(&**pool)
        .await
        .map(|record| record.count > 0)
        .unwrap_or(false);

    if user_exists {
        return HttpResponse::BadRequest().json("User already exists");
    }
    // Get the user's passkeys from the database
    let exclude_credentials = get_user_credentials(email, &pool).await;

    match data.start_passkey_registration(user_unique_id, email, display_name, exclude_credentials)
    {
        Ok((challenge_response, passkey_registration)) => {
            store_passkey_registration(email, display_name, &passkey_registration, &pool).await;

            // Send the challenge to the client
            HttpResponse::Ok().json(challenge_response)
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to start registration"),
    }
}
