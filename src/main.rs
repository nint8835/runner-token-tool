use chrono::{Duration, Utc};
use clap::{crate_version, AppSettings, Clap, ValueHint};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

mod resp_types;

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

struct Error(String);

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();

    let private_key_bytes = std::fs::read(opts.private_key_path.to_owned())
        .map_err(|e| Error(format!("Unable to read private key file: {}", e)))?;

    let private_key = EncodingKey::from_rsa_pem(&private_key_bytes)
        .map_err(|e| Error(format!("Unable to create RSA encoding key: {}", e)))?;

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
            iss: opts.app_id.to_owned(),
        },
        &private_key,
    )
    .expect("Failed to create JWT");

    let client = reqwest::blocking::Client::new();

    let installations: Vec<resp_types::Installation> = client
        .get("https://api.github.com/app/installations")
        .header("Accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("Bearer {}", app_token))
        .header(
            "User-Agent",
            format!(
                "runner-token-tool/{} (https://github.com/nint8835/runner-token-tool)",
                crate_version!()
            ),
        )
        .send()
        .map_err(|e| Error(format!("Failed to request app installations: {}", e)))?
        .json()
        .map_err(|e| {
            Error(format!(
                "Failed to deserialize installations response: {}",
                e
            ))
        })?;

    let installation = installations
        .iter()
        .find(|&installation| match &installation.account {
            resp_types::Account::User(org) => &org.login == &opts.org_name,
            _ => false,
        })
        .ok_or(Error(
            "No installation matching that organization name could be found.".to_string(),
        ))?;

    let installation_token: resp_types::TokenResp = client
        .post(installation.access_tokens_url.to_owned())
        .header("Accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("Bearer {}", app_token))
        .header(
            "User-Agent",
            format!(
                "runner-token-tool/{} (https://github.com/nint8835/runner-token-tool)",
                crate_version!()
            ),
        )
        .send()
        .map_err(|e| Error(format!("Failed to request installation token: {}", e)))?
        .json()
        .map_err(|e| Error(format!("Failed to deserialize token response: {}", e)))?;

    let token_endpoint = match opts.token_type {
        TokenType::REGISTRATION => "actions/runners/registration-token",
        TokenType::REMOVAL => "actions/runners/remove-token",
    };

    let token_resp: resp_types::TokenResp = client
        .post(format!(
            "https://api.github.com/orgs/{org_name}/{endpoint}",
            org_name = opts.org_name,
            endpoint = token_endpoint
        ))
        .header("Accept", "application/vnd.github.v3+json")
        .header(
            "Authorization",
            format!("Bearer {}", installation_token.token),
        )
        .header(
            "User-Agent",
            format!(
                "runner-token-tool/{} (https://github.com/nint8835/runner-token-tool)",
                crate_version!()
            ),
        )
        .send()
        .map_err(|e| Error(format!("Failed to request runner token: {}", e)))?
        .json()
        .map_err(|e| {
            Error(format!(
                "Failed to deserialize runner token response: {}",
                e
            ))
        })?;

    println!("{}", token_resp.token);

    Ok(())
}
