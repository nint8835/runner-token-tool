# runner-token-tool

Tool for automatically generating tokens for managing GitHub Actions Self-hosted Runners, without the usage of machine users or Personal Access Tokens.

## Installation

### Installation from source

1. Create and activate a new virtualenv.
   - `python3 -m venv venv`
   - `source venv/bin/activate`
   - You can also use an alternate virtualenv management tool, such as [pyenv-virtualenv](https://github.com/pyenv/pyenv-virtualenv).
2. Install Poetry.
   - `pip install poetry`
3. Install `runner-token-tool`.
   - `poetry install`

## Configuration

1. Create a new GitHub App from the developer settings in either your account, or the target organization.
   - The name, callback url, etc. do not matter.
   - In the permission section, under Organization permissions, set the access for Self-hosted runners to Read & Write.
   - If created in your own account, ensure it is set to allow any account to install it.
2. Once created, make note of the "App ID" value near the top of the page.
3. Scroll down to Private keys and generate a new key.
   - When prompted, download the private key file.
4. In the left sidebar, select Install App, then follow the instructions from there to install it on your target organization.

## Usage

```
runner-token-tool TOKEN_TYPE PRIVATE_KEY_PATH APP_ID ORG_NAME
```

| Argument name      | Description                                                        |
| ------------------ | ------------------------------------------------------------------ |
| `TOKEN_TYPE`       | The type of token to generate, either `registration` or `removal`. |
| `PRIVATE_KEY_PATH` | The path to the private key for your GitHub App.                   |
| `APP_ID`           | The ID of your GitHub App.                                         |
| `ORG_NAME`         | The name of your target organization.                              |

### Example

```
runner-token-tool registration ./runner-token-tool.2021-05-17.private-key.pem 111111 my-org-name
```

## Limitations

- This tool does not currently support generating tokens for installing runners on individual repositories.
  - The repository runner management API routes require repository administrator permissions to use, whereas the organization API routes support using a much more restricted permission scope for management.
  - The best alternative if you wish to restrict runners per-repo is to configure [runner groups](https://docs.github.com/en/actions/hosting-your-own-runners/managing-access-to-self-hosted-runners-using-groups) for each repository.
  - Alternatively, you can use [labels](https://docs.github.com/en/actions/hosting-your-own-runners/using-labels-with-self-hosted-runners) as a way to filter on runners, but this provides no protection from a repo just using the label of another repo's runner.
