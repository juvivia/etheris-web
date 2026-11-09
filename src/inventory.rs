    view! {
        <section class="inventory-section">
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

            {selected_details}
        </section>
    }
}
