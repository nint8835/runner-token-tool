from datetime import datetime, timedelta
from enum import Enum
from pathlib import Path
from typing import Dict

import jwt
import typer
from requests import Session

app = typer.Typer()

requests_session = Session()
requests_session.headers.update({"Accept": "application/vnd.github.v3+json"})


class TokenType(Enum):
    REGISTRATION = "registration"
    REMOVAL = "removal"


TOKEN_ENDPOINTS: Dict[TokenType, str] = {
    TokenType.REGISTRATION: "actions/runners/registration-token",
    TokenType.REMOVAL: "actions/runners/remove-token",
}


@app.command()
def get_token(
    token_type: TokenType = typer.Argument(..., help="Type of token to generate."),
    private_key_path: Path = typer.Argument(
        ..., help="Path to file containing your GitHub App's private key."
    ),
    app_id: str = typer.Argument(
        ..., help="ID of the GitHub App to be used to generate tokens."
    ),
    org_name: str = typer.Argument(
        ..., help="ID of the organization to generate tokens for."
    ),
) -> None:
    """Generate a token to perform a self-hosted runner operation."""
    with open(private_key_path) as private_key_file:
        private_key = private_key_file.read()

    app_jwt = jwt.encode(
        {
            "iat": datetime.utcnow() - timedelta(seconds=10),
            "exp": datetime.utcnow() + timedelta(minutes=10),
            "iss": app_id,
        },
        private_key,
        "RS256",
    )

    installations_resp = requests_session.get(
        "https://api.github.com/app/installations",
        headers={"Authorization": f"Bearer {app_jwt}"},
    )
    installations_resp.raise_for_status()

    access_tokens_url = ""

    for installation in installations_resp.json():
        if installation["account"]["login"] == org_name:
            access_tokens_url = installation["access_tokens_url"]
            break
    else:
        typer.secho(
            "No installation matching that organization ID could be found.", fg="red"
        )
        raise typer.Exit(1)

    installation_token_resp = requests_session.post(
        access_tokens_url, headers={"Authorization": f"Bearer {app_jwt}"}
    )
    installation_token_resp.raise_for_status()
    installation_token = installation_token_resp.json()["token"]

    token_resp = requests_session.post(
        f"https://api.github.com/orgs/{org_name}/{TOKEN_ENDPOINTS[token_type]}",
        headers={"Authorization": f"Bearer {installation_token}"},
    )
    token_resp.raise_for_status()
    token = token_resp.json()["token"]

    typer.echo(token)


app()
