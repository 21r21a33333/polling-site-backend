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
    option_id: String, // Add option_id to the claims
    exp: usize, // Expiration time in seconds
}

#[derive(Deserialize)]
struct FinishAuthenticationRequest {
    email: String,
    public_key_credential: PublicKeyCredential,
    option_id: String, // Include option_id in the request
}

#[post("/getpass")]
pub async fn finish_verification(
    pool: web::Data<MySqlPool>, // Your MySQL connection pool
    req_body: web::Json<FinishAuthenticationRequest>,
) -> impl Responder {
    println!("/POST login/finish");
    
    let data = create_webauthn_instance();
    let email = &req_body.email;
    let public_key_credential = &req_body.public_key_credential;
    let option_id = &req_body.option_id; // Extract option_id from the request

    // Retrieve the passkey authentication state from the database
    let passkey_auth_state = get_passkey_auth_state(email, &pool).await;

    // Finish the WebAuthn authentication
    match data.finish_passkey_authentication(public_key_credential, &passkey_auth_state) {
        Ok(auth_result) => {
            // Optionally handle auth_result if needed

            // Update credential counter
            update_credential_counter(email, 1, &pool).await;

            // Create JWT claims with email and option_id
            let my_claims = Claims {
                sub: email.to_owned(),
                option_id: option_id.to_owned(), // Include option_id
                exp: 10000000000, // Set expiration time here
            };
            let secret_key = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
            let token = encode(
                &Header::default(),
                &my_claims,
                &EncodingKey::from_secret(secret_key.as_ref()), // Secret key for signing
            ).expect("Failed to encode token"); // Handle errors appropriately

            HttpResponse::Ok()
                .json(serde_json::json!({ "token": token , "message": "Authentication successful"}))
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to finish authentication"),
    }
}