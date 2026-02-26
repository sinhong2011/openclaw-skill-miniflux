use miniflux_api::MinifluxApi;
use reqwest::Client;
use url::Url;

pub enum Auth {
    Token(String),
    UserPass { username: String, password: String },
}

pub struct Config {
    pub url: Url,
    pub auth: Auth,
    pub read_only: bool,
}

impl Config {
    pub fn new(
        url: String,
        api_token: Option<String>,
        username: Option<String>,
        password: Option<String>,
        read_only: bool,
    ) -> Result<Self, String> {
        let url = Url::parse(&url).map_err(|e| format!("Invalid URL: {e}"))?;

        let auth = match (api_token, username, password) {
            (Some(token), _, _) => Auth::Token(token),
            (None, Some(user), Some(pass)) => Auth::UserPass {
                username: user,
                password: pass,
            },
            (None, Some(_), None) | (None, None, Some(_)) => {
                return Err("Both --username and --password are required for user/pass auth".into());
            }
            (None, None, None) => {
                return Err(
                    "Authentication required: provide --api-token or --username + --password"
                        .into(),
                );
            }
        };

        Ok(Config {
            url,
            auth,
            read_only,
        })
    }

    pub fn create_api(&self) -> MinifluxApi {
        match &self.auth {
            Auth::Token(token) => MinifluxApi::new_from_token(&self.url, token.clone()),
            Auth::UserPass { username, password } => {
                MinifluxApi::new(&self.url, username.clone(), password.clone())
            }
        }
    }

    pub fn create_client() -> Client {
        Client::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_auth_valid() {
        let config = Config::new(
            "http://localhost:8080".into(),
            Some("mytoken".into()),
            None,
            None,
            false,
        );
        assert!(config.is_ok());
    }

    #[test]
    fn test_userpass_auth_valid() {
        let config = Config::new(
            "http://localhost:8080".into(),
            None,
            Some("admin".into()),
            Some("pass".into()),
            false,
        );
        assert!(config.is_ok());
    }

    #[test]
    fn test_no_auth_fails() {
        let config = Config::new(
            "http://localhost:8080".into(),
            None,
            None,
            None,
            false,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_partial_userpass_fails() {
        let config = Config::new(
            "http://localhost:8080".into(),
            None,
            Some("admin".into()),
            None,
            false,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_read_only_flag() {
        let config = Config::new(
            "http://localhost:8080".into(),
            Some("token".into()),
            None,
            None,
            true,
        )
        .unwrap();
        assert!(config.read_only);
    }
}
