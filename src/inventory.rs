use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Representa um item colocado no inventário.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct InventoryItem {
    pub(crate) id: u32,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) name: String,
    pub(crate) color: String,
}

pub(crate) fn default_inventory_width() -> u8 {
    6
}

pub(crate) fn default_inventory_height() -> u8 {
    6
}

/// Verifica se um item cabe no grid e se não sobrepõe outro item.
fn inventory_can_place(
    items: &[InventoryItem],
    x: u8,
    y: u8,
    item_width: u8,
    item_height: u8,
    grid_width: u8,
    grid_height: u8,
) -> bool {
    if x.saturating_add(item_width) > grid_width
        || y.saturating_add(item_height) > grid_height
    {
        return false;
    }

    !items.iter().any(|item| {
        x < item.x + item.width
            && x + item_width > item.x
            && y < item.y + item.height
            && y + item_height > item.y
    })
}

/// Componente principal do inventário.
#[component]
pub(crate) fn Inventory(sheet: RwSignal<crate::Sheet>) -> impl IntoView {
    let selected_size = RwSignal::new("1x1".to_string());
    let selected_item = RwSignal::new(None::<u32>);

    // Cria as células vazias do grid.
    let grid_cells = move || {
        let current = sheet.get();
        let width = current.inventory_width;
        let height = current.inventory_height;

        (0..(width as usize * height as usize))
            .map(|index| {
                let x = (index % width as usize) as u8;
                let y = (index / width as usize) as u8;

                view! {
                    <button
                        type="button"
                        class="inventory-cell"
                        style=format!(
                            "grid-column: {}; grid-row: {};",
                            x + 1,
                            y + 1
                        )
                        aria-label=format!(
                            "Espaço {}, {}",
                            x + 1,
                            y + 1
                        )
                        on:click=move |_| {
                            let (iw, ih) =
                                match selected_size.get_untracked().as_str() {
                                    "1x2" => (2, 1),
                                    "2x2" => (2, 2),
                                    "3x3" => (3, 3),
                                    _ => (1, 1),
                                };

                            let mut new_id = None;

                            sheet.update(|s| {
                                if inventory_can_place(
                                    &s.inventory_items,
                                    x,
                                    y,
                                    iw,
                                    ih,
                                    s.inventory_width,
                                    s.inventory_height,
                                ) {
                                    let id = s
                                        .inventory_items
                                        .iter()
                                        .map(|i| i.id)
                                        .max()
                                        .unwrap_or(0)
                                        + 1;

                                    s.inventory_items.push(InventoryItem {
                                        id,
                                        x,
                                        y,
                                        width: iw,
                                        height: ih,
                                        name: String::new(),
                                        color: "#536b60".to_string(),
                                    });

                                    new_id = Some(id);
                                }
                            });

                            if let Some(id) = new_id {
                                selected_item.set(Some(id));
                            }
                        }
                    ></button>
                }
            })
            .collect_view()
    };

    // Desenha os itens que já estão no inventário.
    let placed_items = move || {
        sheet
            .get()
            .inventory_items
            .into_iter()
            .map(|item| {
                let item_id = item.id;
                let label = if item.name.trim().is_empty() {
                    "Item".to_string()
                } else {
                    item.name.clone()
                };
                let title = label.clone();

                let style = format!(
                    "grid-column: {} / span {}; grid-row: {} / span {}; background: {};",
                    item.x + 1,
                    item.width,
                    item.y + 1,
                    item.height,
                    item.color
                );

                view! {
                    <button
                        type="button"
                        class="inventory-item"
                        style=style
                        title=title
                        on:click=move |_| {
                            selected_item.set(Some(item_id));
                        }
                    >
                        {label}
                    </button>
                }
            })
            .collect_view()
    };

    // Editor do item selecionado.
    let selected_details = move || {
        let item = selected_item.get().and_then(|wanted| {
            sheet
                .get()
                .inventory_items
                .into_iter()
                .find(|item| item.id == wanted)
        });

        match item {
            Some(item) => {
                let item_id = item.id;

                view! {
                    <div class="inventory-editor">
                        <div class="section-heading">
                            <h3>"Editar item"</h3>

                            <button
                                type="button"
                                class="remove"
                                on:click=move |_| {
                                    selected_item.set(None);
                                }
                            >
                                "Fechar"
                            </button>
                        </div>

                        <label>
                            "Nome do item"

                            <input
                                type="text"
                                placeholder="Ex.: Kit médico"
                                prop:value=item.name.clone()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);

                                    sheet.update(|s| {
                                        if let Some(i) = s
                                            .inventory_items
                                            .iter_mut()
                                            .find(|i| i.id == item_id)
                                        {
                                            i.name = value;
                                        }
                                    });
                                }
                            />
                        </label>

                        <label>
                            "Cor do item"

                            <input
                                class="inventory-color"
                                type="color"
                                prop:value=item.color.clone()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);

                                    sheet.update(|s| {
                                        if let Some(i) = s
                                            .inventory_items
                                            .iter_mut()
                                            .find(|i| i.id == item_id)
                                        {
                                            i.color = value;
                                        }
                                    });
                                }
                            />
                        </label>

                        <button
                            type="button"
                            class="remove inventory-delete"
                            on:click=move |_| {
                                sheet.update(|s| {
                                    s.inventory_items.retain(|i| i.id != item_id);
                                });

                                selected_item.set(None);
                            }
                        >
                            "Remover item"
                        </button>
                    </div>
                }
                .into_any()
            }

            None => {
                view! {
                    <p class="hint">
                        "Clique em um item para editar seu nome ou sua cor."
                    </p>
                }
                .into_any()
            }
        }
    };

    view! {
        <section class="inventory-section">
            <div class="section-heading">
                <h3>"Inventário"</h3>
            </div>

            <p class="hint">
                "Defina largura e altura. Escolha o tamanho e clique em um espaço vazio para adicionar um item."
            </p>

            <div class="inventory-settings">
                <label>
                    "Largura"

                    <input
                        type="number"
                        min="1"
                        max="12"
                        prop:value=move || {
                            sheet.get().inventory_width.to_string()
                        }
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                let v = v.clamp(1, 12);

                                sheet.update(|s| {
                                    if s.inventory_items.iter().all(|i| i.x + i.width <= v) {
                                        s.inventory_width = v;
                                    }
                                });
                            }
                        }
                    />
                </label>

                <label>
                    "Altura"

                    <input
                        type="number"
                        min="1"
                        max="12"
                        prop:value=move || {
                            sheet.get().inventory_height.to_string()
                        }
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                let v = v.clamp(1, 12);

                                sheet.update(|s| {
                                    if s.inventory_items.iter().all(|i| i.y + i.height <= v) {
                                        s.inventory_height = v;
                                    }
                                });
                            }
                        }
                    />
                </label>

                <label>
                    "Tamanho do próximo item"

                    <select
                        prop:value=move || selected_size.get()
                        on:change=move |ev| {
                            selected_size.set(event_target_value(&ev));
                        }
                    >
                        <option value="1x1">"Pequeno — 1×1"</option>
                        <option value="1x2">"Médio — 2×1"</option>
                        <option value="2x2">"Grande — 2×2"</option>
                        <option value="3x3">"Gigantesco — 3×3"</option>
                    </select>
                </label>
            </div>

        <div
    class="inventory-grid"
    style=move || {
        let current = sheet.get();

        format!(
            "grid-template-columns: repeat({}, minmax(0, 1fr)); grid-template-rows: repeat({}, minmax(0, 1fr)); aspect-ratio: {} / {};",
            current.inventory_width,
            current.inventory_height,
            current.inventory_width,
            current.inventory_height
        )
    }
>
        
                {grid_cells}
                {placed_items}
            </div>

            {selected_details}
        </section>
    }
}
