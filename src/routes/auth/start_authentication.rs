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




#[derive(Serialize,Deserialize)]
pub struct StartAuthenticationRequest {
   pub email: String,
}

#[post("/login/start")]
pub async fn start_authentication(
    pool: web::Data<sqlx::MySqlPool>, // Your MySQL connection pool
    req_body: web::Json<StartAuthenticationRequest>,
) -> impl Responder {
    println!("POST /login/start");
    let data = create_webauthn_instance();
    let email = &req_body.email;

    // Retrieve the user's credentials from the database
    let user_passkeys = get_user_credentials_passkeys(email, &pool).await;
    let user_passkeys = match user_passkeys {
        None => return HttpResponse::BadRequest().json("User not found"),
        Some(user_passkeys) => user_passkeys,
    };

    // Start WebAuthn authentication
    match data.start_passkey_authentication(&user_passkeys) {
        Ok((challenge_response, passkey_auth_state)) => {
            // Persist the `passkey_auth_state` for this user
            store_passkey_auth_state(email, &passkey_auth_state, &pool).await;

            // Send the challenge to the client
            HttpResponse::Ok().json(challenge_response)
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to start authentication"),
    }
}
