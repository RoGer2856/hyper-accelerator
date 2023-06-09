use std::sync::Arc;

use crate::{
    application_context_trait::ApplicationContextTrait,
    prelude::ResultInspector,
    request_context_trait::RequestContextTrait,
    request_handler::{
        ErrorResponse, Request, RequestHandlerFn, RequestHandlerReturnTrait, Response,
    },
    response::create_empty_response,
};

pub enum AuthenticatorError {
    InvalidCredentials,
    InvalidAccessToken,
    InvalidHttpHeaderValue,
    InternalError,
}

impl From<AuthenticatorError> for ErrorResponse {
    fn from(e: AuthenticatorError) -> Self {
        let status = match e {
            AuthenticatorError::InvalidCredentials => hyper::StatusCode::BAD_REQUEST,
            AuthenticatorError::InvalidAccessToken => hyper::StatusCode::BAD_REQUEST,
            AuthenticatorError::InvalidHttpHeaderValue => hyper::StatusCode::BAD_REQUEST,
            AuthenticatorError::InternalError => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        };

        create_empty_response(status).into()
    }
}

impl From<hyper::header::InvalidHeaderValue> for AuthenticatorError {
    fn from(_e: hyper::header::InvalidHeaderValue) -> Self {
        AuthenticatorError::InvalidHttpHeaderValue
    }
}

pub trait AuthenticatorApplicationContext {
    fn update_access_token(&self, access_token: String) -> Result<String, AuthenticatorError>;
    fn verify_access_token(&self, access_token: &str) -> Result<(), AuthenticatorError>;
}

pub trait AuthenticatorRequestContext {
    fn set_verified_access_token(&mut self, access_token: &str);
}

pub fn add_access_token_to_resp(
    access_token: String,
    resp: &mut Response,
) -> Result<(), AuthenticatorError> {
    let header_name = "Set-Cookie";
    let cookie = cookie::CookieBuilder::new("access_token", access_token)
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .finish();
    let header_value = hyper::header::HeaderValue::from_str(&cookie.to_string())
        .inspect_err(|_| log::error!("Cannot convert access_token to header value"))?;
    resp.headers_mut().insert(header_name, header_value);
    Ok(())
}

pub async fn access_token_handler<
    ApplicationContextType: ApplicationContextTrait + AuthenticatorApplicationContext,
    RequestContextType: RequestContextTrait + AuthenticatorRequestContext,
    NextReturnType: RequestHandlerReturnTrait,
>(
    req: Request,
    app_context: Arc<ApplicationContextType>,
    mut request_context: RequestContextType,
    next: impl RequestHandlerFn<ApplicationContextType, RequestContextType, NextReturnType>,
) -> Result<Response, ErrorResponse> {
    let mut access_token = None;
    for cookie in crate::cookies::cookies_iter(crate::cookies::CookieType::Cookie, req.headers()) {
        if cookie.name() == "access_token" && !crate::cookies::is_cookie_expired_by_date(&cookie) {
            let at = cookie.value().to_string();
            if let Ok(()) = app_context.verify_access_token(&at) {
                access_token = Some(at);
            }
        }
    }

    let access_token = if let Some(access_token) = access_token {
        request_context.set_verified_access_token(&access_token);
        Some(access_token)
    } else {
        None
    };

    let ret = next(req, app_context.clone(), request_context).await;

    if let Some(access_token) = access_token {
        let access_token = app_context.update_access_token(access_token)?;

        match ret {
            Ok(mut resp) => {
                add_access_token_to_resp(access_token, &mut resp)?;
                Ok(resp)
            }
            Err(mut resp) => {
                add_access_token_to_resp(access_token, &mut resp.0)?;
                Err(resp)
            }
        }
    } else {
        ret
    }
}
