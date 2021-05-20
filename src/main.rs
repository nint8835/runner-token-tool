use chrono::{Duration, Utc};
use clap::{crate_version, AppSettings, Clap, ValueHint};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clap, PartialEq)]
enum TokenType {
    /// Token for registering a new self-hosted runner.
    REGISTRATION,
    /// Token for removing an existing self-hosted runner.
    REMOVAL,
}

/// Generate a token to perform a self-hosted runner operation.
#[derive(Clap, Debug)]
#[clap(version = crate_version!(), author = "nint8835 <riley@rileyflynn.me>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Type of token to generate.
    #[clap(arg_enum)]
    token_type: TokenType,
    /// Path to the file containing your GitHub App's private key.
    #[clap(parse(from_os_str), value_hint = ValueHint::FilePath)]
    private_key_path: PathBuf,
    /// ID of the GitHub App to be used to generate tokens.
    app_id: String,
    /// Name of the GitHub organization to generate tokens for.
    org_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppTokenClaims {
    exp: usize,
    iat: usize,
    iss: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    let private_key = match std::fs::read(opts.private_key_path) {
        Ok(private_key_bytes) => {
            EncodingKey::from_rsa_pem(&private_key_bytes).expect("Unable to create encoding key")
        }
        Err(e) => {
            eprintln!("Unable to read private key file: {}", e);
            std::process::exit(1);
        }
    };

    let app_token = encode(
        &Header::new(Algorithm::RS256),
        &AppTokenClaims {
            exp: Utc::now()
                .checked_add_signed(Duration::minutes(10))
                .unwrap()
                .timestamp() as usize,
            iat: Utc::now()
                .checked_sub_signed(Duration::seconds(10))
                .unwrap()
                .timestamp() as usize,
            iss: opts.app_id,
        },
        &private_key,
    )
    .expect("Failed to create JWT");

    println!("Generated JWT: {}", app_token)
}
