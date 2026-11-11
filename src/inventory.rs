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

fn inventory_size_for_selection(size: &str) -> Option<(u8, u8)> {
    match size {
        "1x1" => Some((1, 1)),
        "2x1" => Some((2, 1)),
        "2x2" => Some((2, 2)),
        "3x3" => Some((3, 3)),
        _ => None,
    }
}

fn inventory_can_place(
    items: &[InventoryItem],
    x: u8,
    y: u8,
    item_width: u8,
    item_height: u8,
    grid_width: u8,
    grid_height: u8,
    ignore_id: Option<u32>,
) -> bool {
    if x.saturating_add(item_width) > grid_width
        || y.saturating_add(item_height) > grid_height
    {
        return false;
    }

    !items.iter().any(|item| {
        if ignore_id == Some(item.id) {
            return false;
        }

        x < item.x + item.width
            && x + item_width > item.x
            && y < item.y + item.height
            && y + item_height > item.y
    })
}

/// Tenta usar a célula indicada como âncora e testa os outros cantos possíveis.
/// Isso permite colocar ou soltar itens médios clicando em qualquer metade e
/// também ajuda itens grandes perto das bordas do inventário.
fn inventory_find_placement(
    items: &[InventoryItem],
    x: u8,
    y: u8,
    item_width: u8,
    item_height: u8,
    grid_width: u8,
    grid_height: u8,
    ignore_id: Option<u32>,
) -> Option<(u8, u8)> {
    for y_offset in 0..item_height {
        for x_offset in 0..item_width {
            let Some(place_x) = x.checked_sub(x_offset) else {
                continue;
            };
            let Some(place_y) = y.checked_sub(y_offset) else {
                continue;
            };

            if inventory_can_place(
                items,
                place_x,
                place_y,
                item_width,
                item_height,
                grid_width,
                grid_height,
                ignore_id,
            ) {
                return Some((place_x, place_y));
            }
        }
    }

    None
}

fn move_inventory_item(sheet: RwSignal<crate::Sheet>, item_id: u32, x: u8, y: u8) {
    sheet.update(|s| {
        let item_size = s
            .inventory_items
            .iter()
            .find(|item| item.id == item_id)
            .map(|item| (item.width, item.height));

        if let Some((item_width, item_height)) = item_size {
            if let Some((new_x, new_y)) = inventory_find_placement(
                &s.inventory_items,
                x,
                y,
                item_width,
                item_height,
                s.inventory_width,
                s.inventory_height,
                Some(item_id),
            ) {
                if let Some(item) = s.inventory_items.iter_mut().find(|item| item.id == item_id) {
                    item.x = new_x;
                    item.y = new_y;
                }
            }
        }
    });
}

fn rotate_inventory_item(sheet: RwSignal<crate::Sheet>, item_id: u32) {
    sheet.update(|s| {
        let Some(item) = s.inventory_items.iter().find(|item| item.id == item_id) else {
            return;
        };
        let (x, y, width, height) = (item.x, item.y, item.height, item.width);

        if let Some((new_x, new_y)) = inventory_find_placement(
            &s.inventory_items,
            x,
            y,
            width,
            height,
            s.inventory_width,
            s.inventory_height,
            Some(item_id),
        ) {
            if let Some(item) = s.inventory_items.iter_mut().find(|item| item.id == item_id) {
                item.x = new_x;
                item.y = new_y;
                item.width = width;
                item.height = height;
            }
        }
    });
}

fn css_px(value: Option<String>) -> f64 {
    value
        .as_deref()
        .unwrap_or("0")
        .trim_end_matches("px")
        .parse::<f64>()
        .unwrap_or(0.0)
}

fn coordinate_to_cell(
    coordinate: f64,
    start: f64,
    end: f64,
    leading_border: f64,
    leading_padding: f64,
    trailing_padding: f64,
    trailing_border: f64,
    gap: f64,
    tracks: u8,
) -> u8 {
    let content_start = start + leading_border + leading_padding;
    let content_end = end - trailing_border - trailing_padding;
    let track_space = (content_end - content_start - gap * f64::from(tracks.saturating_sub(1)))
        .max(1.0);
    let track_size = track_space / f64::from(tracks.max(1));
    let step = track_size + gap;
    let position = ((coordinate - content_start) / step).floor();
    position.clamp(0.0, f64::from(tracks.saturating_sub(1))) as u8
}

/// Componente principal do inventário.
#[component]
pub(crate) fn Inventory(sheet: RwSignal<crate::Sheet>) -> impl IntoView {
    let selected_size = RwSignal::new("1x1".to_string());
    let selected_item = RwSignal::new(None::<u32>);
    let dragged_item = RwSignal::new(None::<u32>);
    let grid_ref = NodeRef::<leptos::html::Div>::new();

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
                        style=format!("grid-column: {}; grid-row: {};", x + 1, y + 1)
                        aria-label=format!("Espaço {}, {}", x + 1, y + 1)
                        on:click=move |_| {
                            // Alternativa de toque para dispositivos sem drag-and-drop nativo.
                            if let Some(item_id) = selected_item.get_untracked() {
                                move_inventory_item(sheet, item_id, x, y);
                                return;
                            }

                            let (item_width, item_height) = inventory_size_for_selection(
                                &selected_size.get_untracked(),
                            )
                            .unwrap_or((1, 1));
                            let mut new_id = None;

                            sheet.update(|s| {
                                if let Some((place_x, place_y)) = inventory_find_placement(
                                    &s.inventory_items,
                                    x,
                                    y,
                                    item_width,
                                    item_height,
                                    s.inventory_width,
                                    s.inventory_height,
                                    None,
                                ) {
                                    let id = s.inventory_items.iter()
                                        .map(|item| item.id)
                                        .max()
                                        .unwrap_or(0)
                                        .saturating_add(1);
                                    s.inventory_items.push(InventoryItem {
                                        id,
                                        x: place_x,
                                        y: place_y,
                                        width: item_width,
                                        height: item_height,
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

    let placed_items = move || {
        sheet.get().inventory_items.into_iter().map(|item| {
            let item_id = item.id;
            let item_width = item.width;
            let item_height = item.height;
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
                <div
                    class="inventory-item"
                    class:inventory-item-selected=move || selected_item.get() == Some(item_id)
                    style=style
                    title=title
                    role="button"
                    tabindex="0"
                    draggable="true"
                    on:click=move |_| selected_item.set(Some(item_id))
                    on:dragstart=move |ev| {
                        dragged_item.set(Some(item_id));
                        if let Some(data_transfer) = ev.data_transfer() {
                            let _ = data_transfer.set_effect_allowed("move");
                            let _ = data_transfer.set_data("text/plain", &item_id.to_string());
                        }
                    }
                    on:dragend=move |_| dragged_item.set(None)
                >
                    {label}
                    <Show when=move || selected_item.get() == Some(item_id) && item_width != item_height>
                        <button
                            type="button"
                            class="inventory-rotate"
                            aria-label="Girar item"
                            title="Girar item"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                rotate_inventory_item(sheet, item_id);
                            }
                        >
                            "↻"
                        </button>
                    </Show>
                </div>
            }
        }).collect_view()
    };

    let selected_details = move || {
        let item = selected_item.get().and_then(|wanted| {
            sheet.get().inventory_items.into_iter().find(|item| item.id == wanted)
        });

        match item {
            Some(item) => {
                let item_id = item.id;
                view! {
                    <div class="inventory-editor">
                        <div class="section-heading">
                            <h3>"Editar item"</h3>
                            <button type="button" class="remove" on:click=move |_| selected_item.set(None)>
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
                                        if let Some(i) = s.inventory_items.iter_mut().find(|i| i.id == item_id) {
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
                                        if let Some(i) = s.inventory_items.iter_mut().find(|i| i.id == item_id) {
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
                                sheet.update(|s| s.inventory_items.retain(|i| i.id != item_id));
                                selected_item.set(None);
                            }
                        >
                            "Remover item"
                        </button>
                    </div>
                }.into_any()
            }
            None => view! {
                <p class="hint">"Clique em um item para editar seu nome ou sua cor."</p>
            }.into_any(),
        }
    };

    view! {
        <section class="inventory-section">
            <div class="inventory-main">
            <div class="section-heading"><h3>"Inventário"</h3></div>
            <p class="hint">
                "Escolha o tamanho e clique em um espaço vazio para adicionar. Arraste um item ou toque nele e depois na célula de destino para movê-lo. O botão ↻ gira itens médios."
            </p>
            <div class="inventory-settings">
                <div class="inventory-dimensions">
                    <label class="inventory-dimension-field">
                        <span class="inventory-dimension-label">"Largura"</span>
                        <input
                            type="number" min="1" max="12"
                            prop:value=move || sheet.get().inventory_width.to_string()
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

                    <span class="inventory-dimensions-separator">"×"</span>

                    <label class="inventory-dimension-field">
                        <span class="inventory-dimension-label">"Altura"</span>
                        <input
                            type="number" min="1" max="12"
                            prop:value=move || sheet.get().inventory_height.to_string()
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
                </div>

                <label>
                    "Tamanho do próximo item"
                    <select
                        prop:value=move || selected_size.get()
                        on:change=move |ev| selected_size.set(event_target_value(&ev))
                    >
                        <option value="1x1">"Pequeno — 1×1"</option>
                        <option value="2x1">"Médio — 2×1"</option>
                        <option value="2x2">"Grande — 2×2"</option>
                        <option value="3x3">"Gigantesco — 3×3"</option>
                    </select>
                </label>
            </div>

            <div
                node_ref=grid_ref
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
                on:dragover=move |ev| ev.prevent_default()
                on:drop=move |ev| {
                    ev.prevent_default();
                    let Some(item_id) = dragged_item.get_untracked() else { return; };
                    let Some(element) = grid_ref.get() else { return; };
                    let rect = element.get_bounding_client_rect();
                    let Some(window) = web_sys::window() else { return; };
                    let Ok(Some(style)) = window.get_computed_style(&element) else { return; };
                    let current = sheet.get_untracked();
                    let x = coordinate_to_cell(
                        f64::from(ev.client_x()),
                        rect.left(),
                        rect.right(),
                        css_px(style.get_property_value("border-left-width").ok()),
                        css_px(style.get_property_value("padding-left").ok()),
                        css_px(style.get_property_value("padding-right").ok()),
                        css_px(style.get_property_value("border-right-width").ok()),
                        css_px(style.get_property_value("column-gap").ok()),
                        current.inventory_width,
                    );
                    let y = coordinate_to_cell(
                        f64::from(ev.client_y()),
                        rect.top(),
                        rect.bottom(),
                        css_px(style.get_property_value("border-top-width").ok()),
                        css_px(style.get_property_value("padding-top").ok()),
                        css_px(style.get_property_value("padding-bottom").ok()),
                        css_px(style.get_property_value("border-bottom-width").ok()),
                        css_px(style.get_property_value("row-gap").ok()),
                        current.inventory_height,
                    );
                    move_inventory_item(sheet, item_id, x, y);
                    dragged_item.set(None);
                }
            >
                {grid_cells}
                {placed_items}
            </div>
            </div>

            {selected_details}
        </section>
    }
}
