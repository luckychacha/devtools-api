use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    TypedHeader,
};
use jwt::Validation;
use serde::{Deserialize, Serialize};

use super::http_error::HttpError;
use jsonwebtoken as jwt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: usize,
    pub name: String,
    pub exp: usize,
}

const SECRET: &[u8] = b"secret";

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bear)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| HttpError::Auth)?;

        let key = jwt::DecodingKey::from_secret(SECRET);
        let token =
            jwt::decode::<Claims>(bear.token(), &key, &Validation::default()).map_err(|e| {
                println!("{e:?}");
                HttpError::Internal
            })?;

        println!("{token:?}");
        Ok(token.claims)
    }
}
