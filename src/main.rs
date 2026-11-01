use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Item {
    name: String,
    description: String,
    weight: u8,
    condition: String,
    cards: u8,
}

#[derive(Clone, Serialize, Deserialize)]
struct Skill {
    name: String,
    description: String,
}

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
    abilities: Vec<Skill>,
    capacity: u8,
    items: Vec<Item>,
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
            abilities: Vec::new(),
            capacity: 3,
            items: Vec::new(),
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

#[component]
fn App() -> impl IntoView {
    let sheet = RwSignal::new(load_sheet());
    let tab = RwSignal::new(0usize);
    let skill_name = RwSignal::new(String::new());
    let skill_description = RwSignal::new(String::new());

    Effect::new(move |_| save_sheet(&sheet.get()));

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
                    "Anotações"
                </button>
            </nav>

            <section class="content">
                <Show when=move || tab.get() == 0>
                    <h2>"Personagem"</h2>

                    <div class="form-grid compact-grid">
                        <label class="compact-field">
                            "Nome"
                            <input prop:value=move || sheet.get().name.clone() on:input=move |ev| sheet.update(|s| s.name = event_target_value(&ev)) />
                        </label>

                        <label class="compact-field">
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

                        <label class="compact-field">
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

                        <label class="compact-field">
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

                        <label class="compact-field">
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

                        <label class="compact-field">
                            "Nível"
                            <input type="number" min="1" max="10" prop:value=move || sheet.get().level.to_string() on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                    sheet.update(|s| s.level = v.clamp(1, 10));
                                }
                            } />
                        </label>
                    </div>

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

                    <h3>"Habilidades"</h3>

                    <Show when=move || sheet.get().abilities.is_empty()>
                        <p class="hint">"Nenhuma habilidade adicionada ainda."</p>
                    </Show>

                    <Show when=move || !sheet.get().abilities.is_empty()>
                        <div class="skill-stack">
                            {move || {
                                sheet.get().abilities.iter().enumerate().map(|(index, ability)| {
                                    let ability_name = ability.name.clone();
                                    let ability_description = ability.description.clone();
                                    view! {
                                        <div class="skill-card">
                                            <div class="skill-head">
                                                <strong>{ability_name}</strong>
                                                <button class="remove" on:click=move |_| {
                                                    sheet.update(|s| {
                                                        if index < s.abilities.len() {
                                                            s.abilities.remove(index);
                                                        }
                                                    });
                                                }>
                                                    "Remover"
                                                </button>
                                            </div>
                                            <p>{if ability_description.is_empty() {
                                                "Sem descrição registrada.".to_string()
                                            } else {
                                                ability_description
                                            }}</p>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </Show>

                    <div class="skill-form">
                        <label class="compact-field">
                            "Nome da habilidade"
                            <input placeholder="Ex.: Domínio do fogo" prop:value=move || skill_name.get() on:input=move |ev| skill_name.set(event_target_value(&ev)) />
                        </label>
                        <label class="compact-field">
                            "Descrição"
                            <textarea rows="2" placeholder="Descreva a habilidade" prop:value=move || skill_description.get() on:input=move |ev| skill_description.set(event_target_value(&ev))></textarea>
                        </label>
                    </div>

                    <div class="skill-actions">
                        <span class="pill">{move || format!("{}/8", sheet.get().abilities.len())}</span>
                        <button class="primary" on:click=move |_| {
                            let name = skill_name.get().trim().to_string();
                            let description = skill_description.get();
                            if !name.is_empty() && sheet.get().abilities.len() < 8 {
                                sheet.update(|s| {
                                    s.abilities.push(Skill { name, description });
                                });
                                skill_name.set(String::new());
                                skill_description.set(String::new());
                            }
                        }>
                            "+ Adicionar habilidade"
                        </button>
                    </div>
                </Show>

                <Show when=move || tab.get() == 1>
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
