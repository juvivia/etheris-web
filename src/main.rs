use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Item {
    name: String,
    condition: String,
    advantage: String,
    cards: u8,
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
    abilities: String,
    capacity: u8,
    items: Vec<Item>,
    notes: String,
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            name: "Novo personagem".into(),
            race: "".into(),
            specialization: "".into(),
            community: "".into(),
            class_name: "".into(),
            level: 1,
            pv: 0,
            san: 0,
            pm: 0,
            stats: [0; 5],
            abilities: String::new(),
            capacity: 3,
            items: vec![],
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

#[component]
fn App() -> impl IntoView {
    let sheet = RwSignal::new(load_sheet());
    let tab = RwSignal::new(0usize);
    let item_name = RwSignal::new(String::new());
    let item_advantage = RwSignal::new(String::new());
    Effect::new(move |_| save_sheet(&sheet.get()));
    let labels = ["INT", "FIS", "DOM", "VEL", "HAB"];

    let inventory_items = move || {
        sheet
            .get()
            .items
            .iter()
            .cloned()
            .enumerate()
            .collect::<Vec<_>>()
    };

    view! {
        <main class="shell">
            <header class="topbar">
                <div><h1>"ETHERIS"</h1><p>"THE GAME · FICHA DE PERSONAGEM"</p></div>
                <span class="level">"NÍVEL " {move || sheet.get().level}</span>
            </header>

            <nav class="tabs" aria-label="Seções da ficha">
                <button class:active={move || tab.get() == 0} on:click={move |_| tab.set(0)}>
                    "Personagem"
                </button>
                <button class:active={move || tab.get() == 1} on:click={move |_| tab.set(1)}>
                    "Inventário"
                </button>
                <button class:active={move || tab.get() == 2} on:click={move |_| tab.set(2)}>
                    "Anotações"
                </button>
            </nav>

            <section class="content">
                <Show when={move || tab.get() == 0}>
                    <h2>"Personagem"</h2>
                    <div class="form-grid">
                        <label>
                            "Nome"
                            <input prop:value={move || sheet.get().name} on:input={move |ev| sheet.update(|s| s.name = event_target_value(&ev))} />
                        </label>
                        <label>
                            "Raça"
                            <input placeholder="Raça" prop:value={move || sheet.get().race} on:input={move |ev| sheet.update(|s| s.race = event_target_value(&ev))} />
                        </label>
                        <label>
                            "Especialização"
                            <input prop:value={move || sheet.get().specialization} on:input={move |ev| sheet.update(|s| s.specialization = event_target_value(&ev))} />
                        </label>
                        <label>
                            "Comunidade"
                            <input prop:value={move || sheet.get().community} on:input={move |ev| sheet.update(|s| s.community = event_target_value(&ev))} />
                        </label>
                        <label>
                            "Classe"
                            <input prop:value={move || sheet.get().class_name} on:input={move |ev| sheet.update(|s| s.class_name = event_target_value(&ev))} />
                        </label>
                        <label>
                            "Nível"
                            <input type="number" min="1" max="10" prop:value={move || sheet.get().level.to_string()} on:input={move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                    sheet.update(|s| s.level = v.clamp(1, 10));
                                }
                            }} />
                        </label>
                    </div>

                    <h3>"Recursos"</h3>
                    <div class="resource-grid">
                        <label>
                            "PV"
                            <input type="number" prop:value={move || sheet.get().pv.to_string()} on:input={move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.pv = v);
                                }
                            }} />
                        </label>
                        <label>
                            "SAN"
                            <input type="number" prop:value={move || sheet.get().san.to_string()} on:input={move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.san = v);
                                }
                            }} />
                        </label>
                        <label>
                            "PM"
                            <input type="number" prop:value={move || sheet.get().pm.to_string()} on:input={move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    sheet.update(|s| s.pm = v);
                                }
                            }} />
                        </label>
                    </div>

                    <h3>"Atributos"</h3>
                    <div class="stat-grid">
                        {labels.iter().enumerate().map(|(i, label)| {
                            let label = *label;
                            view! {
                                <div class="stat">
                                    <strong>{label}</strong>
                                    <input
                                        aria-label={format!("Atributo {label}")}
                                        type="number"
                                        prop:value={move || sheet.get().stats[i].to_string()}
                                        on:input={move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                sheet.update(|s| s.stats[i] = v);
                                            }
                                        }}
                                    />
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <h3>"Habilidades"</h3>
                    <textarea
                        rows="5"
                        placeholder="Habilidades de classe, especialização e comunidade..."
                        prop:value={move || sheet.get().abilities}
                        on:input={move |ev| sheet.update(|s| s.abilities = event_target_value(&ev))}
                    ></textarea>
                </Show>

                <Show when={move || tab.get() == 1}>
                    <div class="section-heading">
                        <h2>"Inventário"</h2>
                        <span class="pill">
                            {move || format!("{} / {} cartas", sheet.get().items.iter().map(|i| i.cards as u16).sum::<u16>(), sheet.get().capacity as u16)}
                        </span>
                    </div>

                    <label class="capacity">
                        "Capacidade em cartas"
                        <input type="number" min="1" max="99" prop:value={move || sheet.get().capacity.to_string()} on:input={move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                sheet.update(|s| s.capacity = v.max(1));
                            }
                        }} />
                    </label>

                    <div class="item-list">
                        {move || inventory_items().into_iter().map(|(index, item)| {
                            let item_name_value = item.name.clone();
                            let item_condition = item.condition.clone();
                            let item_advantage = item.advantage.clone();
                            let item_cards = item.cards;
                            view! {
                                <div class="item">
                                    <div class="item-title">
                                        <strong>{item_name_value}</strong>
                                        <button class="remove" on:click={move |_| {
                                            sheet.update(|s| {
                                                if index < s.items.len() {
                                                    s.items.remove(index);
                                                }
                                            });
                                        }}>
                                            "Remover"
                                        </button>
                                    </div>
                                    <p>{format!("Estado: {} · {} carta(s)", item_condition, item_cards)}</p>
                                    <p class="advantage">
                                        {if item_advantage.is_empty() {
                                            "Sem vantagem registrada".to_string()
                                        } else {
                                            format!("Vantagem em teste: {}", item_advantage)
                                        }}
                                    </p>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <h3>"Adicionar item"</h3>
                    <div class="form-grid">
                        <label>
                            "Item"
                            <input placeholder="Nome do item" prop:value={move || item_name.get()} on:input={move |ev| item_name.set(event_target_value(&ev))} />
                        </label>
                        <label>
                            "Vantagem em teste (opcional)"
                            <input placeholder="Ex.: Pontaria" prop:value={move || item_advantage.get()} on:input={move |ev| item_advantage.set(event_target_value(&ev))} />
                        </label>
                    </div>

                    <button class="primary" on:click={move |_| {
                        let name = item_name.get().trim().to_string();
                        if !name.is_empty() && sheet.get().items.iter().map(|i| i.cards as u16).sum::<u16>() < sheet.get().capacity as u16 {
                            let advantage = item_advantage.get();
                            sheet.update(|s| {
                                s.items.push(Item {
                                    name,
                                    condition: "Íntegro".into(),
                                    advantage,
                                    cards: 1,
                                });
                            });
                            item_name.set(String::new());
                            item_advantage.set(String::new());
                        }
                    }}>
                        "+ Adicionar carta"
                    </button>

                    <p class="hint">"Nesta versão, cada item ocupa 1 carta. Bônus e estados são registros manuais."</p>
                </Show>

                <Show when={move || tab.get() == 2}>
                    <h2>"Anotações"</h2>
                    <p class="hint">"Pistas, acontecimentos e objetivos da campanha."</p>
                    <textarea
                        class="notes"
                        rows="16"
                        placeholder="Escreva aqui..."
                        prop:value={move || sheet.get().notes}
                        on:input={move |ev| sheet.update(|s| s.notes = event_target_value(&ev))}
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
