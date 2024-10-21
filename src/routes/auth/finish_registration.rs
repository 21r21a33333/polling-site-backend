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
struct FinishRegistrationRequest {
    email: String,
    public_key_credential: RegisterPublicKeyCredential,
}

#[post("/register/finish")]
pub async fn finish_registration(
    pool: web::Data<sqlx::MySqlPool>, // Your MySQL connection pool
    req_body: web::Json<FinishRegistrationRequest>,
) -> impl Responder {
    println!("/POST register/finish");

    let data = create_webauthn_instance();
    let email = &req_body.email;
    let public_key_credential = &req_body.public_key_credential;

    // Retrieve the passkey registration state from the database
    let passkey_registration = get_passkey_registration(email, &pool).await.unwrap();

    // Finish the WebAuthn registration
    match data.finish_passkey_registration(&public_key_credential, &passkey_registration) {
        Ok(auth_result) => {
            // Store the new credential and user
            store_user_credential(email, &auth_result, &pool).await;

            HttpResponse::Ok().json("Registration successful")
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to finish registration"),
    }
}
