use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use reqwest::{Error, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Pool;
use sqlx_postgres::PgPool;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::str::FromStr;
use tokio::sync::Mutex;

#[derive(Deserialize, Serialize)]
struct TokenResponse {
    #[serde(rename = "iamToken")]
    iam_token: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

pub struct TokenInfo {
    token: String,
    expired_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl TokenInfo {
    pub fn from_file(name: &str) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(name)
            .expect("token.json could not be opened");
        let mut buff = String::new();

        file.read_to_string(&mut buff)
            .expect("token.json could not be read");

        let now = Utc::now();

        if let Ok(token) = serde_json::from_str::<TokenResponse>(&buff) {
            let datetime = NaiveDateTime::from_str(&token.expires_at)
                .map_or_else(|_| now, |date| date.and_utc());

            TokenInfo {
                token: token.iam_token,
                expired_at: datetime,
                created_at: now,
            }
        } else {
            TokenInfo {
                token: "".to_string(),
                expired_at: now,
                created_at: now,
            }
        }
    }
    pub fn new(token: String, expired_at: DateTime<Utc>, created_at: DateTime<Utc>) -> Self {
        let token = TokenInfo {
            token,
            expired_at,
            created_at,
        };

        let json = serde_json::to_string_pretty::<TokenResponse>(&TokenResponse {
            iam_token: token.token.clone(),
            expires_at: token.expired_at.to_string(),
        })
        .unwrap();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("./token.json")
            .expect("token.json could not be opened");

        file.write_all(json.as_bytes()).unwrap();

        token
    }

    pub fn is_expired(&self) -> bool {
        let one_hour = Duration::hours(1);
        let now = Utc::now();

        self.created_at + one_hour < now
    }

    pub fn get_token(&self) -> String {
        self.token.clone()
    }
}

pub struct State {
    pub url: Url,
    pub pool: PgPool,
    pub token_info: Mutex<TokenInfo>,
    pub folder_id: String,
    oauth_token: String,
}

impl State {
    pub fn new(
        ya_cloud_url: &str,
        pool: PgPool,
        oauth_token: String,
        folder_id: String,
        token_info: TokenInfo,
    ) -> Self {
        let url = Url::parse(&ya_cloud_url).expect("failed to parse ya cloud url");

        State {
            url,
            pool,
            token_info: Mutex::new(token_info),
            folder_id,
            oauth_token,
        }
    }

    pub async fn update_token(&self) -> Result<(), Error> {
        let mut token = self.token_info.lock().await;

        if token.expired_at == token.created_at
            || token.created_at + Duration::hours(1) < Utc::now()
        {
            let reqwest_client = reqwest::Client::new();
            let res = reqwest_client
                .post(self.url.clone())
                .body(json!({ "yandexPassportOauthToken": self.oauth_token }).to_string())
                .send()
                .await?;

            if res.status() == 200 {
                let data = res.json::<TokenResponse>().await?;
                let datetime = &data
                    .expires_at
                    .parse::<DateTime<Utc>>()
                    .expect("Ya send invalid date string");
                let new_token_info = TokenInfo::new(data.iam_token, datetime.to_utc(), Utc::now());
                *token = new_token_info;
            };
        }

        Ok(())
    }
}
