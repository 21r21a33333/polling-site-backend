use std::env;

use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, http::header::AUTHORIZATION, middleware::Next, HttpMessage};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};


// jwt middleware
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Claims {
    sub: String,
    exp: usize,
    // Add other fields as needed
}
pub async fn jwt_middleware(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    println!("JWT middleware called");
    let headers = req.headers();
    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
                let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
                let validation = Validation::new(Algorithm::HS256);

                match decode::<Claims>(token, &decoding_key, &validation) {
                    Ok(token_data) => {
                        let claims = token_data.claims.clone();
                        req.extensions_mut().insert(claims.clone());
                        req.headers_mut().insert(
                            actix_web::http::header::HeaderName::from_static("user_id"),
                            claims.sub.parse().unwrap(),
                        );
                        println!("user_id in req header: {:?}", req.headers().get("user_id"));
                        return next.call(req).await;
                    }
                    Err(_) => {
                        return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
                    }
                }
            }
        }
    } else {
        return Err(actix_web::error::ErrorUnauthorized("No token provided"));
    }
    next.call(req).await
}


