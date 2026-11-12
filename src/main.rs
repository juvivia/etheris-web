use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

mod inventory;
mod auth;
mod cloud;

#[derive(Clone, Serialize, Deserialize)]
struct Sheet {
    name: String,
    race: String,
    specialization: String,
    community: String,
    class_name: String,
    level: u8,
    pv: i32,
    san: i32,
    pm: i32,
    #[serde(default)]
    pv_max: i32,
    #[serde(default)]
    san_max: i32,
    #[serde(default)]
    pm_max: i32,
    stats: [i32; 5],
    abilities: String,
    capacity: u8,
    notes: String,
    #[serde(default = "inventory::default_inventory_width")]
    pub(crate) inventory_width: u8,
    #[serde(default = "inventory::default_inventory_height")]
    pub(crate) inventory_height: u8,
    #[serde(default)]
    pub(crate) inventory_items: Vec<inventory::InventoryItem>,
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            name: "Novo personagem".into(),
            race: String::new(),
            specialization: String::new(),
            community: String::new(),
            class_name: String::new(),
            level: 1,
            pv: 0,
            san: 0,
            pm: 0,
            pv_max: 0,
            san_max: 0,
            pm_max: 0,
            stats: [0; 5],
            abilities: String::new(),
            capacity: 3,
            notes: String::new(),
            inventory_width: 6,
            inventory_height: 6,
            inventory_items: Vec::new(),
        }
    }
}

async fn prepare_account(
    session: auth::AuthSession,
) -> Result<(auth::AuthSession, Sheet, String), String> {
    let character = match cloud::load_character(&session).await? {
        Some(character) => character,
        None => {
            let default_sheet = Sheet::default();
            let data = serde_json::to_value(&default_sheet)
                .map_err(|e| e.to_string())?;

            cloud::create_character(
                &session,
                &default_sheet.name,
                data,
            )
            .await?
        }
    };

    let sheet = serde_json::from_value::<Sheet>(character.sheet_data)
        .unwrap_or_default();

    Ok((session, sheet, character.id))
}

fn load_sheet() -> Sheet {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("etheris-sheet-v1").ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_sheet(sheet: &Sheet) {
    if let (Some(storage), Ok(json)) = (
        web_sys::window().and_then(|w| w.local_storage().ok().flatten()),
        serde_json::to_string(sheet),
    ) {
        let _ = storage.set_item("etheris-sheet-v1", &json);
    }
}

fn specialization_options_for_race(race: &str) -> Vec<&'static str> {
    match race {
        "Humano" => vec!["Amalgamos", "Flagelos", "Aplicadores", "Suplícios"],
        "Áureo" => vec!["Místico", "Corpóreo"],
        "Pesadelo" => vec!["Rei", "Dama", "Valete"],
        _ => vec![],
    }
}

fn encode_skill_description(description: &str) -> String {
    description.replace('\n', "\\n")
}

fn decode_skill_description(description: &str) -> String {
    description.replace("\\n", "\n")
}

#[component]
fn SheetApp(
    initial_sheet: Sheet,
    session: auth::AuthSession,
    character_id: String,
    on_logout: Callback<()>,
) -> impl IntoView {
    let sheet = RwSignal::new(initial_sheet);
    let save_status = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let tab = RwSignal::new(0usize);
    let resource_editing = RwSignal::new(false);
    let skill_name = RwSignal::new(String::new());
    let skill_description = RwSignal::new(String::new());
    let skill_form_open = RwSignal::new(false);

    Effect::new(move |_| save_sheet(&sheet.get()));

    let skill_items = move || {
        let raw = sheet.get().abilities;

        if let Ok(items) = serde_json::from_str::<Vec<(String, String)>>(&raw) {
            return items;
        }

        raw.lines()
            .map(str::trim)
            .filter(|skill| !skill.is_empty())
            .filter_map(|skill| {
                if let Some((name, description)) = skill.split_once("::") {
                    let name = name.trim();
                    if name.is_empty() {
                        None
                    } else {
                        Some((
                            name.to_string(),
                            decode_skill_description(description.trim()),
                        ))
                    }
                } else {
                    Some((skill.to_string(), String::new()))
                }
            })
            .collect::<Vec<_>>()
    };

    let add_skill = move |_| {
        let name = skill_name.get().trim().to_string();
        let description = skill_description.get().trim().to_string();

        if name.is_empty() {
            return;
        }

        let mut items = skill_items();
        if !items.iter().any(|(existing_name, _)| {
            existing_name.eq_ignore_ascii_case(&name)
        }) {
            items.push((name, description));
        }

        sheet.update(|s| {
            s.abilities = serde_json::to_string(&items).unwrap_or_default();
        });

        skill_name.set(String::new());
        skill_description.set(String::new());
    };

    let remove_skill = move |skill_to_remove: String| {
        let items = skill_items()
            .into_iter()
            .filter(|(name, _)| name != &skill_to_remove)
            .collect::<Vec<_>>();

        sheet.update(|s| {
            s.abilities = serde_json::to_string(&items).unwrap_or_default();
        });
    };

    let submit_skill = move |_| {
        if !skill_form_open.get() {
            skill_form_open.set(true);
            return;
        }

        let has_name = !skill_name.get().trim().is_empty();
        if has_name {
            add_skill(());
            skill_form_open.set(false);
        }
    };

    let render_skill_list = move || {
        let items = skill_items();

        if items.is_empty() {
            view! {
                <p class="skill-empty">"Nenhuma habilidade adicionada."</p>
            }
            .into_any()
        } else {
            view! {
                <div class="item-list">
                    {items.into_iter().map(|(skill_title, skill_text)| {
                        let skill_title_clone = skill_title.clone();

                        view! {
                            <div class="item">
                                <div class="item-title">
                                    <strong>{skill_title}</strong>
                                    <button
                                        type="button"
                                        class="remove"
                                        on:click=move |_| remove_skill(skill_title_clone.clone())
                                    >
                                        "Remover"
                                    </button>
                                </div>
                                <p>
                                    {if skill_text.is_empty() {
                                        "Nenhuma descrição registrada.".to_string()
                                    } else {
                                        skill_text
                                    }}
                                </p>
                            </div>
                        }
                    }).collect_view()}
                </div>
            }
            .into_any()
        }
    };

    view! {
        <main class="shell">
            <header class="topbar">
                <div>
                    <h1>"SAMSARA"</h1>
                    <p>"Character Sheet Manager"</p>
                </div>
                <span class="level">"NÍVEL " {move || sheet.get().level}</span>
            </header>

            <nav class="tabs" aria-label="Seções da ficha">
                <button class:active=move || tab.get() == 0 on:click=move |_| tab.set(0)>
                    "Personagem"
                </button>
                <button class:active=move || tab.get() == 1 on:click=move |_| tab.set(1)>
                    "Atributos"
                </button>
                <button class:active=move || tab.get() == 2 on:click=move |_| tab.set(2)>
                    "Anotações"
                </button>
            </nav>

            <section class="content">
                <Show when=move || tab.get() == 0>
                    <h2>"Personagem"</h2>
                    <div class="form-grid">
                        <label>
                            "Nome"
                            <input prop:value=move || sheet.get().name.clone() on:input=move |ev| sheet.update(|s| s.name = event_target_value(&ev)) />
                        </label>
                        <label>
                            "Raça"
                            <select
                                prop:value=move || sheet.get().race.clone()
                                on:change=move |ev| {
                                    let selected = event_target_value(&ev);
                                    sheet.update(|s| {
                                        s.race = selected.clone();
                                        let opts = specialization_options_for_race(&selected);
                                        if opts.is_empty() {
                                            s.specialization.clear();
                                        } else if !opts.iter().any(|opt| *opt == s.specialization.as_str()) {
                                            s.specialization = opts[0].to_string();
                                        }
                                    });
                                }
                            >
                                <option value="" selected=move || sheet.get().race.is_empty() >"Selecione"</option>
                                <option value="Humano" selected=move || sheet.get().race == "Humano">"Humano"</option>
                                <option value="Pesadelo" selected=move || sheet.get().race == "Pesadelo">"Pesadelo"</option>
                                <option value="Áureo" selected=move || sheet.get().race == "Áureo">"Áureo"</option>
                            </select>
                        </label>
                        <label>
                            "Especialização"
                            <select
                                prop:value=move || sheet.get().specialization.clone()
                                on:change=move |ev| sheet.update(|s| s.specialization = event_target_value(&ev))
                            >
                                <option value="" selected=move || sheet.get().specialization.is_empty() >"Selecione"</option>
                                {move || {
                                    specialization_options_for_race(&sheet.get().race)
                                        .into_iter()
                                        .map(|opt| {
                                            let selected = sheet.get().specialization == opt;
                                            view! { <option value={opt} selected={selected}>{opt}</option> }
                                        })
                                        .collect_view()
                                }}
                            </select>
                        </label>
                        <label>
                            "Comunidade"
                            <select
                                prop:value=move || sheet.get().community.clone()
                                on:change=move |ev| sheet.update(|s| s.community = event_target_value(&ev))
                            >
                                <option value="" selected=move || sheet.get().community.is_empty() >"Selecione"</option>
                                <option value="Pântano" selected=move || sheet.get().community == "Pântano">"Pântano"</option>
                                <option value="Tundra" selected=move || sheet.get().community == "Tundra">"Tundra"</option>
                                <option value="Cidade" selected=move || sheet.get().community == "Cidade">"Cidade"</option>
                                <option value="Nômade" selected=move || sheet.get().community == "Nômade">"Nômade"</option>
                                <option value="Deserto" selected=move || sheet.get().community == "Deserto">"Deserto"</option>
                            </select>
                        </label>
                        <label>
                            "Classe"
                            <select
                                prop:value=move || sheet.get().class_name.clone()
                                on:change=move |ev| sheet.update(|s| s.class_name = event_target_value(&ev))
                            >
                                <option value="" selected=move || sheet.get().class_name.is_empty() >"Selecione"</option>
                                <option value="Médico" selected=move || sheet.get().class_name == "Médico">"Médico"</option>
                                <option value="Artífice" selected=move || sheet.get().class_name == "Artífice">"Artífice"</option>
                                <option value="Engenheiro" selected=move || sheet.get().class_name == "Engenheiro">"Engenheiro"</option>
                                <option value="Magus" selected=move || sheet.get().class_name == "Magus">"Magus"</option>
                                <option value="Cozinheiro" selected=move || sheet.get().class_name == "Cozinheiro">"Cozinheiro"</option>
                            </select>
                        </label>
                        <label>
                            "Nível"
                            <input type="number" min="1" max="10" prop:value=move || sheet.get().level.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                    sheet.update(|s| s.level = v.clamp(1, 10));
                                }
                            } />
                        </label>
                    </div>

                    <h3>"Habilidades"</h3>
                    <div class="skill-field">
                        <div class="skill-form">
                            <Show when=move || skill_form_open.get()>
                                <input
                                    class="skill-input"
                                    placeholder="Nome da habilidade"
                                    prop:value=move || skill_name.get()
                                    on:input=move |ev| skill_name.set(event_target_value(&ev))
                                />
                                <div class="skill-description-row">
                                    <textarea
                                        class="skill-input skill-description-input"
                                        rows="3"
                                        placeholder="Descrição"
                                        prop:value=move || skill_description.get()
                                        on:input=move |ev| skill_description.set(event_target_value(&ev))
                                    ></textarea>
                                </div>
                            </Show>
                        </div>
                        <button type="button" class="skill-add" on:click=submit_skill>
                            {move || if skill_form_open.get() { "Salvar" } else { "Adicionar" }}
                        </button>
                        <div class="skill-list">{render_skill_list}</div>
                    </div>
                </Show>

                <Show when=move || tab.get() == 1>
                    <h2>"Atributos"</h2>
                    <div class="resource-heading">
                        <h3>"Recursos"</h3>
                    </div>
                    <div class="resource-grid">
                        <label class="resource-card">
                            "PV"
                            <input type="number" aria-label="PV atual" prop:value=move || sheet.get().pv.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.pv = v); }
                            } />
                            <input class="resource-max-input" type="number" min="0" disabled=move || !resource_editing.get() prop:value=move || {
                                let s = sheet.get();
                                if s.pv_max > 0 { s.pv_max } else { s.pv }.to_string()
                            } on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.pv_max = v.max(0)); }
                            } />
                        </label>
                        <label class="resource-card">
                            "SAN"
                            <input type="number" aria-label="SAN atual" prop:value=move || sheet.get().san.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.san = v); }
                            } />
                            <input class="resource-max-input" type="number" min="0" disabled=move || !resource_editing.get() prop:value=move || {
                                let s = sheet.get();
                                if s.san_max > 0 { s.san_max } else { s.san }.to_string()
                            } on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.san_max = v.max(0)); }
                            } />
                        </label>
                        <label class="resource-card">
                            "PM"
                            <input type="number" aria-label="PM atual" prop:value=move || sheet.get().pm.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.pm = v); }
                            } />
                            <input class="resource-max-input" type="number" min="0" disabled=move || !resource_editing.get() prop:value=move || {
                                let s = sheet.get();
                                if s.pm_max > 0 { s.pm_max } else { s.pm }.to_string()
                            } on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.pm_max = v.max(0)); }
                            } />
                        </label>
                    </div>

                    <h3>"Atributos"</h3>
                    <div class="stat-grid">
                        <div class="stat"><strong>"INT"</strong><input aria-label="Atributo INT" type="number" disabled=move || !resource_editing.get() prop:value=move || sheet.get().stats[0].to_string() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.stats[0] = v); } } /></div>
                        <div class="stat"><strong>"FIS"</strong><input aria-label="Atributo FIS" type="number" disabled=move || !resource_editing.get() prop:value=move || sheet.get().stats[1].to_string() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.stats[1] = v); } } /></div>
                        <div class="stat"><strong>"DOM"</strong><input aria-label="Atributo DOM" type="number" disabled=move || !resource_editing.get() prop:value=move || sheet.get().stats[2].to_string() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.stats[2] = v); } } /></div>
                        <div class="stat"><strong>"VEL"</strong><input aria-label="Atributo VEL" type="number" disabled=move || !resource_editing.get() prop:value=move || sheet.get().stats[3].to_string() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.stats[3] = v); } } /></div>
                        <div class="stat"><strong>"HAB"</strong><input aria-label="Atributo HAB" type="number" disabled=move || !resource_editing.get() prop:value=move || sheet.get().stats[4].to_string() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { sheet.update(|s| s.stats[4] = v); } } /></div>
                    </div>
                </Show>

                <Show when=move || tab.get() == 2>
                    <div class="notes-panel">
                        <h2>"Anotações"</h2>
                        <p class="hint">"Escreva.."</p>
                        <textarea class="notes" rows="23" placeholder="Escreva aqui..." prop:value=move || sheet.get().notes.clone() on:input=move |ev| sheet.update(|s| s.notes = event_target_value(&ev))></textarea>
                    </div>
                    <inventory::Inventory sheet=sheet />
                </Show>
            </section>

            <footer class="account-footer">
                <div class="account-footer-actions">
                    <Show when=move || tab.get() == 1 && !resource_editing.get()>
                        <button
                            type="button"
                            class="resource-edit"
                            disabled=move || saving.get()
                            on:click=move |_| {
                                sheet.update(|s| {
                                    if s.pv_max <= 0 { s.pv_max = s.pv; }
                                    if s.san_max <= 0 { s.san_max = s.san; }
                                    if s.pm_max <= 0 { s.pm_max = s.pm; }
                                });
                                resource_editing.set(true);
                                save_status.set("Edite os valores máximos e os atributos e clique em Salvar ficha.".to_string());
                            }
                        >
                            "Editar máximos"
                        </button>
                    </Show>

                    <button
                        type="button"
                        class="resource-edit"
                        disabled=move || saving.get()
                        on:click=move |_| {
                            let current_sheet = sheet.get();
                            let current_session = session.clone();
                            let current_id = character_id.clone();
                            let name = current_sheet.name.clone();
                            let data = match serde_json::to_value(&current_sheet) {
                                Ok(value) => value,
                                Err(error) => { save_status.set(format!("Erro: {}", error)); return; }
                            };
                            saving.set(true);
                            save_status.set("Salvando...".to_string());
                            spawn_local(async move {
                                let result = cloud::update_character(&current_session, &current_id, &name, data).await;
                                match result {
                                    Ok(()) => {
                                        save_status.set("Ficha salva na nuvem.".to_string());
                                        resource_editing.set(false);
                                    }
                                    Err(error) => save_status.set(format!("Erro ao salvar: {}", error)),
                                }
                                saving.set(false);
                            });
                        }
                    >
                        {move || if saving.get() { "Salvando..." } else { "Salvar ficha" }}
                    </button>

                    <button
                        type="button"
                        class="resource-edit"
                        on:click=move |_| on_logout.run(())
                    >
                        "Sair da conta"
                    </button>
                </div>
                <span>{move || save_status.get()}</span>
            </footer>
        </main>
    }
}

#[component]
fn App() -> impl IntoView {
    let account = RwSignal::new(None::<(auth::AuthSession, Sheet, String)>);
    let loading = RwSignal::new(true);
    let busy = RwSignal::new(false);
    let register_mode = RwSignal::new(false);
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error_message = RwSignal::new(String::new());

    Effect::new(move |_| {
        spawn_local(async move {
            if auth::load_session().is_some() {
                match auth::refresh_session().await {
                    Ok(session) => match prepare_account(session).await {
                        Ok(data) => account.set(Some(data)),
                        Err(error) => error_message.set(error),
                    },
                    Err(error) => error_message.set(error),
                }
            }
            loading.set(false);
        });
    });

    let submit_auth = move |_| {
        if busy.get_untracked() { return; }
        let user = username.get_untracked();
        let pass = password.get_untracked();
        let registering = register_mode.get_untracked();
        busy.set(true);
        error_message.set(String::new());

        spawn_local(async move {
            let result = if registering { auth::register(&user, &pass).await } else { auth::login(&user, &pass).await };
            match result {
                Ok(session) => match prepare_account(session).await {
                    Ok(data) => account.set(Some(data)),
                    Err(error) => error_message.set(error),
                },
                Err(error) => error_message.set(error),
            }
            busy.set(false);
        });
    };

    let logout = Callback::new(move |_| {
        spawn_local(async move {
            let _ = auth::logout().await;
            account.set(None);
            username.set(String::new());
            password.set(String::new());
            error_message.set(String::new());
        });
    });

    view! {
        <Show
            when=move || !loading.get()
            fallback=|| view! { <main class="auth-screen"><p>"Carregando..."</p></main> }
        >
            {move || match account.get() {
                Some((session, initial_sheet, character_id)) => view! {
                    <SheetApp initial_sheet=initial_sheet session=session character_id=character_id on_logout=logout />
                }.into_any(),
                None => view! {
                    <main class="auth-screen">
                        <section class="auth-card">
                            <h1>"SAMSARA"</h1>
                            <p>"Character Sheet Manager"</p>
                            <h2>{move || if register_mode.get() { "Criar conta" } else { "Entrar" }}</h2>
                            <label>
                                "Nome de usuário"
                                <input type="text" placeholder="Seu usuário" autocomplete="username" prop:value=move || username.get() on:input=move |ev| username.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                "Senha"
                                <input type="password" placeholder="Sua senha" autocomplete=move || if register_mode.get() { "new-password" } else { "current-password" } prop:value=move || password.get() on:input=move |ev| password.set(event_target_value(&ev)) />
                            </label>
                            <p class="auth-error">{move || error_message.get()}</p>
                            <button type="button" disabled=move || busy.get() on:click=submit_auth>
                                {move || if busy.get() { "Aguarde..." } else if register_mode.get() { "Cadastrar" } else { "Entrar" }}
                            </button>
                            <button type="button" class="auth-switch" on:click=move |_| {
                                register_mode.update(|v| *v = !*v);
                                error_message.set(String::new());
                            }>
                                {move || if register_mode.get() { "Já tenho uma conta" } else { "Criar uma conta" }}
                            </button>
                        </section>
                    </main>
                }.into_any(),
            }}
        </Show>
    }
}

fn main() {
    mount_to_body(App);
}
