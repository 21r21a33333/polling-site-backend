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

#[derive(Deserialize)]
struct StartAuthenticationRequest {
    email: String,
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
