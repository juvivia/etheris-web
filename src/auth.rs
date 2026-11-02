use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use web_sys::window;

const SUPABASE_URL: &str = env!("SUPABASE_URL");
const SUPABASE_KEY: &str = env!("SUPABASE_ANON_KEY");
const SESSION_KEY: &str = "samsara-session";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user: AuthUser,
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    user: Option<AuthUser>,
}

fn internal_email(username: &str) -> Result<String, String> {
    let username = username.trim().to_lowercase();

    if username.len() < 3 || username.len() > 24 {
        return Err("O nome de usuário deve ter entre 3 e 24 caracteres.".into());
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
    {
        return Err("Use apenas letras, números, ponto, hífen ou underline no usuário.".into());
    }

    Ok(format!(
        "{}@thsamvuovhopfpdbbkvn.supabase.co",
        username
    ))
}

fn save_session(session: &AuthSession) -> Result<(), String> {
    let storage = window()
        .ok_or("Janela do navegador indisponível")?
        .local_storage()
        .map_err(|_| "Não foi possível acessar o armazenamento")?
        .ok_or("Armazenamento local indisponível")?;

    let value = serde_json::to_string(session)
        .map_err(|_| "Erro ao preparar a sessão")?;

    storage
        .set_item(SESSION_KEY, &value)
        .map_err(|_| "Não foi possível salvar a sessão")
}

pub fn load_session() -> Option<AuthSession> {
    let storage = window()?.local_storage().ok()??;
    let value = storage.get_item(SESSION_KEY).ok()??;
    serde_json::from_str(&value).ok()
}

pub fn clear_session() -> Result<(), String> {
    let storage = window()
        .ok_or("Janela do navegador indisponível")?
        .local_storage()
        .map_err(|_| "Não foi possível acessar o armazenamento")?
        .ok_or("Armazenamento local indisponível")?;

    storage
        .remove_item(SESSION_KEY)
        .map_err(|_| "Não foi possível encerrar a sessão")
}

async fn response_error(response: gloo_net::http::Response) -> String {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    format!("Erro {}: {}", status, body)
}

pub async fn register(username: &str, password: &str) -> Result<AuthSession, String> {
    let email = internal_email(username)?;

    if password.len() < 8 {
        return Err("A senha precisa ter pelo menos 8 caracteres.".into());
    }

    let body = json!({
        "email": email,
        "password": password
    });

    let response = Request::post(&format!("{}/auth/v1/signup", SUPABASE_URL))
        .header("apikey", SUPABASE_KEY)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(response_error(response).await);
    }

    let data: AuthResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let session = AuthSession {
        access_token: data.access_token
            .ok_or("O Supabase não retornou uma sessão. Confira se a confirmação de e-mail está desativada.")?,
        refresh_token: data.refresh_token
            .ok_or("O Supabase não retornou o token de atualização.")?,
        user: data.user.ok_or("O Supabase não retornou os dados do usuário.")?,
    };

    let profile = json!({
        "id": session.user.id,
        "username": username.trim().to_lowercase()
    });

    let profile_response = Request::post(&format!("{}/rest/v1/profiles", SUPABASE_URL))
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", &format!("Bearer {}", session.access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&profile)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !profile_response.ok() {
        return Err(response_error(profile_response).await);
    }

    save_session(&session)?;
    Ok(session)
}

pub async fn login(username: &str, password: &str) -> Result<AuthSession, String> {
    let email = internal_email(username)?;

    let body = json!({
        "email": email,
        "password": password
    });

    let response = Request::post(&format!(
        "{}/auth/v1/token?grant_type=password",
        SUPABASE_URL
    ))
    .header("apikey", SUPABASE_KEY)
    .header("Content-Type", "application/json")
    .json(&body)
    .map_err(|e| e.to_string())?
    .send()
    .await
    .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(response_error(response).await);
    }

    let session: AuthSession = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    save_session(&session)?;
    Ok(session)
}

pub async fn refresh_session() -> Result<AuthSession, String> {
    let old_session = load_session()
        .ok_or("Nenhuma sessão encontrada. Faça login novamente.")?;

    let body = json!({
        "refresh_token": old_session.refresh_token
    });

    let response = Request::post(&format!(
        "{}/auth/v1/token?grant_type=refresh_token",
        SUPABASE_URL
    ))
    .header("apikey", SUPABASE_KEY)
    .header("Content-Type", "application/json")
    .json(&body)
    .map_err(|e| e.to_string())?
    .send()
    .await
    .map_err(|e| e.to_string())?;

    if !response.ok() {
        let error = response_error(response).await;
        let _ = clear_session();
        return Err(error);
    }

    let session: AuthSession = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    save_session(&session)?;
    Ok(session)
}

pub async fn logout() -> Result<(), String> {
    if let Some(session) = load_session() {
        let response = Request::post(&format!("{}/auth/v1/logout", SUPABASE_URL))
            .header("apikey", SUPABASE_KEY)
            .header("Authorization", &format!("Bearer {}", session.access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            return Err(response_error(response).await);
        }
    }

    clear_session()
}
