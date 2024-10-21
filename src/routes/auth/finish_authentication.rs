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

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize, // Expiration time in seconds
}

#[derive(Deserialize)]
struct FinishAuthenticationRequest {
    email: String,
    public_key_credential: PublicKeyCredential,
}

#[post("/login/finish")]
pub async fn finish_authentication(
    pool: web::Data<sqlx::MySqlPool>, // Your MySQL connection pool
    req_body: web::Json<FinishAuthenticationRequest>,
) -> impl Responder {
    println!("/POST login/finish");
    let data = create_webauthn_instance();
    let email = &req_body.email;
    let public_key_credential = &req_body.public_key_credential;

    // Retrieve the passkey authentication state from the database
    let passkey_auth_state = get_passkey_auth_state(email, &pool).await;

    // Finish the WebAuthn authentication
    match data.finish_passkey_authentication(public_key_credential, &passkey_auth_state) {
        Ok(auth_result) => {
            update_credential_counter(email, 1, &pool).await;
            let my_claims = Claims {
                sub: email.to_owned(),
                exp: 10000000000, // Set expiration time here
            };
            let secret_key = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
            let token = encode(
                &Header::default(),
                &my_claims,
                &EncodingKey::from_secret(secret_key.as_ref()), // Secret key for signing
            )
            .unwrap(); // Handle errors appropriately
            HttpResponse::Ok()
                .json(serde_json::json!({ "token": token , "message": "Authentication successful"}))
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to finish authentication"),
    }
}
