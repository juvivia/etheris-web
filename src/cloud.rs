use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::AuthSession;

const SUPABASE_URL: &str = env!("SUPABASE_URL");
const SUPABASE_KEY: &str = env!("SUPABASE_ANON_KEY");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCharacter {
    pub id: String,
    pub name: String,
    pub sheet_data: serde_json::Value,
    pub portrait_path: Option<String>,
}

pub async fn load_character(session: &AuthSession) -> Result<Option<CloudCharacter>, String> {
    let url = format!(
        "{}/rest/v1/characters?select=id,name,sheet_data,portrait_path&user_id=eq.{}&campaign_id=is.null&order=created_at.asc&limit=1",
        SUPABASE_URL,
        session.user.id
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", &format!("Bearer {}", session.access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Erro ao carregar ficha ({}): {}", status, body));
    }

    let characters: Vec<CloudCharacter> = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(characters.into_iter().next())
}

pub async fn save_character(
    session: &AuthSession,
    character: &CloudCharacter,
) -> Result<(), String> {
    let body = json!({
        "name": character.name,
        "sheet_data": character.sheet_data,
        "portrait_path": character.portrait_path
    });

    let response = Request::post(&format!("{}/rest/v1/characters", SUPABASE_URL))
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", &format!("Bearer {}", session.access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Erro ao salvar ficha ({}): {}", status, body));
    }

    Ok(())
}

pub async fn update_character(
    session: &AuthSession,
    character_id: &str,
    character: &CloudCharacter,
) -> Result<(), String> {
    let body = json!({
        "name": character.name,
        "sheet_data": character.sheet_data,
        "portrait_path": character.portrait_path
    });

    let url = format!(
        "{}/rest/v1/characters?id=eq.{}&user_id=eq.{}",
        SUPABASE_URL,
        character_id,
        session.user.id
    );

    let response = Request::patch(&url)
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", &format!("Bearer {}", session.access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Erro ao atualizar ficha ({}): {}", status, body));
    }

    Ok(())
}
