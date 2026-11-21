// Glue between Horizon's `Game` and `mod_inventory_sorter` runtime state.
use crate::game::native_hooks::NativeMod;
use crate::game::mod_inventory_sorter as m;
use crate::game::{Game, Screen};
use core::ops::ControlFlow;
use horizon_client::{
    containers::BlockEntity, extras_hud::Corners, hud::HudBatch, inv_sort, items::ItemStack,
    menu::Input, stations::hover_name,
};
use std::collections::BTreeSet;
use std::time::Duration;

pub(super) fn on_frame(game: &mut Game<'_>, elapsed: Duration) {
    let dt = elapsed.as_secs_f32();
    m::with_state(|st| {
        if st.toast_timer > 0.0 {
            st.toast_timer = (st.toast_timer - dt).max(0.0);
        }
    });

    // Turn on Horizon's built-in sort buttons / sort key while this mod is enabled.
    let on = game.native_on(m::ID);
    game.settings.extras.sort = on;
    if on {
        game.settings.extras.sort_key =
            game.native_mod_key(m::ID, m::KEY_SORT, m::KEY_SORT_DEFAULT);
    }
}

/// Replaces Horizon's own sort: merges partial stacks and orders by category, then name.
pub(super) fn on_sort_slots(game: &mut Game<'_>, slots: &[usize]) -> ControlFlow<()> {
    if !game.native_on(m::ID) {
        return ControlFlow::Continue(());
    }
    if game.carried.is_some() || slots.is_empty() {
        return ControlFlow::Break(());
    }
    let views: Vec<Option<ItemStack>> = slots.iter().map(|&s| game.slot_view(s)).collect();
    let clicks = {
        let names = |id: &str| game.pack.names.display_item(id);
        let rank = |stack: &ItemStack| {
            let order = game.creative_screen.order_of(stack).unwrap_or(usize::MAX);
            let display = hover_name(&names, stack);
            m::compute_sort_rank(order, &display, &stack.item)
        };
        let max = |stack: &ItemStack| game.max_stack(stack);
        inv_sort::plan(&views, &rank, &max)
    };
    let changed = !clicks.is_empty();
    for i in clicks {
        game.tweak_click(slots[i], 0, Input::Pickup);
    }
    if changed {
        m::push_toast("[Inventory Sorter] Sorted", m::COLOR_INFO);
    }
    ControlFlow::Break(())
}

/// Slots of the open chest-like container (chest, barrel, shulker box, ender chest).
fn open_container_slots(game: &Game<'_>) -> Vec<usize> {
    if game.screen != Screen::Inventory || game.station.is_some() || game.mount_active() {
        return Vec::new();
    }
    let chest_like = if game.remote.is_some() {
        game.remote_storage_open()
    } else {
        matches!(game.container_kind(), Some(BlockEntity::Chest(_)))
    };
    if !chest_like {
        return Vec::new();
    }
    game.slots()
        .into_iter()
        .filter(|s| m::CONTAINER_SLOTS.contains(s))
        .collect()
}

pub(super) fn on_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(m::ID) {
        return;
    }
    if game.screen != Screen::Inventory || game.recipe_search_focused() {
        return;
    }
    let quick_key = game.native_mod_key(m::ID, m::KEY_QUICK_STACK, m::KEY_QUICK_STACK_DEFAULT);
    if key != quick_key {
        return;
    }

    // Quick-Stack: move inventory stacks whose item already exists in the open container.
    let container = open_container_slots(game);
    if container.is_empty() || game.carried.is_some() {
        return;
    }
    let mut existing = BTreeSet::new();
    for &c_slot in &container {
        if let Some(st) = game.slot_view(c_slot) {
            existing.insert(st.item.to_string());
        }
    }
    if existing.is_empty() {
        return;
    }
    let mut moved = 0u32;
    for inv_slot in m::MAIN_INVENTORY {
        if let Some(st) = game.slot_view(inv_slot) {
            if existing.contains(st.item.as_ref())
                && game.tweak_click(inv_slot, 0, Input::QuickMove)
            {
                moved += 1;
            }
        }
    }
    if moved > 0 {
        m::push_toast(
            format!("[Inventory Sorter] Quick-stacked {moved} stack(s)"),
            m::COLOR_INFO,
        );
    }
}

pub(super) fn on_hud(game: &mut Game<'_>, batch: &mut HudBatch, _corners: &mut Corners) {
    if !game.native_on(m::ID) {
        return;
    }
    let (text, color, timer) =
        m::with_state(|st| (st.toast_text.clone(), st.toast_color, st.toast_timer));
    if timer <= 0.0 || text.is_empty() {
        return;
    }
    let w = game.native_text_width(&text);
    let box_w = w + 10.0;
    let box_h = 14.0;
    let x = ((batch.width - box_w) * 0.5).floor();
    let y = (batch.height - 62.0).max(4.0);
    game.native_fill(batch, [x, y, box_w, box_h], m::BACKGROUND);
    game.native_text(batch, &text, [x + 5.0, y + 3.0], color, true);
}

pub(super) fn on_list(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![
            [
                m::KEY_SORT.into(),
                m::KEY_SORT_LABEL.into(),
                m::KEY_SORT_DEFAULT.into(),
            ],
            [
                m::KEY_QUICK_STACK.into(),
                m::KEY_QUICK_STACK_LABEL.into(),
                m::KEY_QUICK_STACK_DEFAULT.into(),
            ],
        ],
    });
}
