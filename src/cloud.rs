
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use js_sys::Uint8Array;

use crate::auth::AuthSession;

const SUPABASE_URL: &str = env!("SUPABASE_URL");
const SUPABASE_KEY: &str = env!("SUPABASE_ANON_KEY");
const PORTRAIT_BUCKET: &str = "portraits";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCharacter {
    pub id: String,
    pub name: String,
    pub sheet_data: Value,
    pub portrait_path: Option<String>,
}

pub async fn load_character(
    session: &AuthSession,
) -> Result<Option<CloudCharacter>, String> {
    let url = format!(
        "{}/rest/v1/characters?select=id,name,sheet_data,portrait_path&user_id=eq.{}&campaign_id=is.null&order=created_at.asc&limit=1",
        SUPABASE_URL,
        session.user.id
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_KEY)
        .header(
            "Authorization",
            &format!("Bearer {}", session.access_token),
        )
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "Erro ao carregar ficha ({}): {}",
            status, body
        ));
    }

    let characters: Vec<CloudCharacter> = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(characters.into_iter().next())
}

pub async fn create_character(
    session: &AuthSession,
    name: &str,
    sheet_data: Value,
) -> Result<CloudCharacter, String> {
    let body = serde_json::json!({
        "user_id": session.user.id,
        "campaign_id": null,
        "name": name,
        "sheet_data": sheet_data,
        "portrait_path": null
    });

    let response = Request::post(
        &format!("{}/rest/v1/characters", SUPABASE_URL)
    )
        .header("apikey", SUPABASE_KEY)
        .header(
            "Authorization",
            &format!("Bearer {}", session.access_token),
        )
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
        return Err(format!(
            "Erro ao criar ficha ({}): {}",
            status, body
        ));
    }

    let mut characters: Vec<CloudCharacter> = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    characters
        .pop()
        .ok_or_else(|| "O Supabase não retornou a ficha criada.".to_string())
}

pub async fn update_character(
    session: &AuthSession,
    character_id: &str,
    name: &str,
    portrait_path: Option<&str>,
    sheet_data: Value,
) -> Result<(), String> {
    let body = serde_json::json!({
        "name": name,
        "sheet_data": sheet_data,
        "portrait_path": portrait_path
    });

    let url = format!(
        "{}/rest/v1/characters?id=eq.{}&user_id=eq.{}",
        SUPABASE_URL,
        character_id,
        session.user.id
    );

    let response = Request::patch(&url)
        .header("apikey", SUPABASE_KEY)
        .header(
            "Authorization",
            &format!("Bearer {}", session.access_token),
        )
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
        return Err(format!(
            "Erro ao atualizar ficha ({}): {}",
            status, body
        ));
    }

    Ok(())
}


pub async fn upload_portrait(
    session: &AuthSession,
    character_id: &str,
    bytes: &[u8],
    content_type: &str,
    extension: &str,
) -> Result<(String, String), String> {
    let path = format!("{}/{}/portrait.{}", character_id, session.user.id, extension);
    let url = format!("{}/storage/v1/object/{}/{}", SUPABASE_URL, PORTRAIT_BUCKET, path);
    let body: wasm_bindgen::JsValue = Uint8Array::from(bytes).into();

    let response = Request::post(&url)
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", &format!("Bearer {}", session.access_token))
        .header("Content-Type", content_type)
        .header("x-upsert", "true")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Erro no Storage ({}): {}", status, body));
    }

    let signed_url = portrait_signed_url(session, &path).await?;
    Ok((path, signed_url))
}

pub async fn portrait_signed_url(
    session: &AuthSession,
    path: &str,
) -> Result<String, String> {
    let url = format!("{}/storage/v1/object/sign/{}/{}", SUPABASE_URL, PORTRAIT_BUCKET, path);
    let body = serde_json::json!({ "expiresIn": 604800 });

    let response = Request::post(&url)
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", &format!("Bearer {}", session.access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Erro ao gerar link do retrato ({}): {}", status, body));
    }

    let result: Value = response.json().await.map_err(|e| e.to_string())?;
    let signed = result
        .get("signedURL")
        .or_else(|| result.get("signedUrl"))
        .and_then(Value::as_str)
        .ok_or("O Supabase não retornou o link do retrato.")?;

    if signed.starts_with("http://") || signed.starts_with("https://") {
        Ok(signed.to_string())
    } else if signed.starts_with("/storage/v1/") {
        Ok(format!("{}{}", SUPABASE_URL, signed))
    } else if signed.starts_with('/') {
        Ok(format!("{}/storage/v1{}", SUPABASE_URL, signed))
    } else {
        Ok(format!("{}/storage/v1/{}", SUPABASE_URL, signed))
    }
}
