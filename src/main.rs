use leptos::prelude::*;
use serde::{Deserialize, Serialize};
mod auth;


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
    stats: [i32; 5],
    abilities: String,
    capacity: u8,
    notes: String,
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
            stats: [0; 5],
            abilities: String::new(),
            capacity: 3,
            notes: String::new(),
        }
    }
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
fn App() -> impl IntoView {
    let sheet = RwSignal::new(load_sheet());
    let tab = RwSignal::new(0usize);
    let skill_name = RwSignal::new(String::new());
    let skill_description = RwSignal::new(String::new());

    Effect::new(move |_| save_sheet(&sheet.get()));

   
let skill_items = move || {
    let raw = sheet.get().abilities;

    // Primeiro tenta ler o formato novo (JSON).
    if let Ok(items) = serde_json::from_str::<Vec<(String, String)>>(&raw) {
        return items;
    }

    // Se não for JSON, tenta ler o formato antigo.
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
                                        on:click=move |_| {
                                            remove_skill(skill_title_clone.clone());
                                        }
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
                    <p>"O RPG"</p>
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
                                <option value="" selected=move || sheet.get().race.is_empty()>"Selecione"</option>
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
                                <option value="" selected=move || sheet.get().specialization.is_empty()>"Selecione"</option>
                                {move || {
                                    specialization_options_for_race(&sheet.get().race)
                                        .into_iter()
                                        .map(|opt| {
                                            let selected = sheet.get().specialization == opt;
                                            view! {
                                                <option value={opt} selected={selected}>{opt}</option>
                                            }
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
                                <option value="" selected=move || sheet.get().community.is_empty()>"Selecione"</option>
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
                                <option value="" selected=move || sheet.get().class_name.is_empty()>"Selecione"</option>
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
                                <button type="button" class="skill-add" on:click=add_skill>"Adicionar"</button>
                            </div>
                        </div>

                        <div class="skill-list">
                            {render_skill_list}
                        </div>
                    </div>
                </Show>

                <Show when=move || tab.get() == 1>
                    <h2>"Atributos"</h2>

                    <h3>"Recursos"</h3>
                    <div class="resource-grid">
                        <label>
                            "PV"
                            <input type="number" prop:value=move || sheet.get().pv.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.pv = v);
                                }
                            } />
                        </label>
                        <label>
                            "SAN"
                            <input type="number" prop:value=move || sheet.get().san.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.san = v);
                                }
                            } />
                        </label>
                        <label>
                            "PM"
                            <input type="number" prop:value=move || sheet.get().pm.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.pm = v);
                                }
                            } />
                        </label>
                    </div>

                    <h3>"Atributos"</h3>
                    <div class="stat-grid">
                        <div class="stat">
                            <strong>"INT"</strong>
                            <input aria-label="Atributo INT" type="number" prop:value=move || sheet.get().stats[0].to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.stats[0] = v);
                                }
                            } />
                        </div>
                        <div class="stat">
                            <strong>"FIS"</strong>
                            <input aria-label="Atributo FIS" type="number" prop:value=move || sheet.get().stats[1].to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.stats[1] = v);
                                }
                            } />
                        </div>
                        <div class="stat">
                            <strong>"DOM"</strong>
                            <input aria-label="Atributo DOM" type="number" prop:value=move || sheet.get().stats[2].to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.stats[2] = v);
                                }
                            } />
                        </div>
                        <div class="stat">
                            <strong>"VEL"</strong>
                            <input aria-label="Atributo VEL" type="number" prop:value=move || sheet.get().stats[3].to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.stats[3] = v);
                                }
                            } />
                        </div>
                        <div class="stat">
                            <strong>"HAB"</strong>
                            <input aria-label="Atributo HAB" type="number" prop:value=move || sheet.get().stats[4].to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.stats[4] = v);
                                }
                            } />
                        </div>
                    </div>
                </Show>

                <Show when=move || tab.get() == 2>
                    <h2>"Anotações"</h2>
                    <p class="hint">"Pistas, acontecimentos e objetivos da campanha."</p>
                    <textarea
                        class="notes"
                        rows="16"
                        placeholder="Escreva aqui..."
                        prop:value=move || sheet.get().notes.clone()
                        on:input=move |ev| sheet.update(|s| s.notes = event_target_value(&ev))
                    ></textarea>
                </Show>
            </section>

            <footer>"Salvamento local neste navegador · Protótipo sem contas"</footer>
        </main>
    }
}

fn main() {
    mount_to_body(App);
}
