use reqwest::Url;
use webauthn_rs::{Webauthn, WebauthnBuilder};

// Create the WebAuthn instance
pub fn create_webauthn_instance() -> Webauthn {
    let rp_id = "localhost"; // Adjust for your actual RP ID
    let rp_origin = Url::parse("http://localhost:3000").expect("Invalid URL");

    let mut builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");
    let webauthn = builder.build().expect("Invalid configuration");
    // println!("{:?}", webauthn);
    return webauthn;
}
