// Inventory Sorter runtime state & pure logic.
// Compiles both standalone (`cargo test`) and inside `horizon-client` via `horizon-mixins`.

use std::sync::Mutex;

pub const ID: &str = "inventory_sorter";
pub const NAME: &str = "Inventory Sorter";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Merges partial stacks and sorts inventory and containers (R), and quick-stacks matching items into an open container (G).";

pub const KEY_SORT: &str = "sort";
pub const KEY_SORT_LABEL: &str = "Sort Inventory / Container";
pub const KEY_SORT_DEFAULT: &str = "key.keyboard.r";

pub const KEY_QUICK_STACK: &str = "quick_stack";
pub const KEY_QUICK_STACK_LABEL: &str = "Quick-Stack to Container";
pub const KEY_QUICK_STACK_DEFAULT: &str = "key.keyboard.g";

pub const BACKGROUND: u32 = 0x7800_0000;
pub const COLOR_INFO: u32 = 0x0055_FF55;

pub const TOAST_SECONDS: f32 = 2.4;

#[derive(Clone, Debug, Default)]
pub struct RuntimeState {
    pub toast_text: String,
    pub toast_color: u32,
    pub toast_timer: f32,
}

static STATE: Mutex<RuntimeState> = Mutex::new(RuntimeState {
    toast_text: String::new(),
    toast_color: COLOR_INFO,
    toast_timer: 0.0,
});

pub fn with_state<R>(f: impl FnOnce(&mut RuntimeState) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

pub fn push_toast(text: impl Into<String>, color: u32) {
    with_state(|st| {
        st.toast_text = text.into();
        st.toast_color = color;
        st.toast_timer = TOAST_SECONDS;
    });
}

/// Rank tuple expected by `horizon_client::inv_sort::plan`:
/// creative-tab order first, then lowercase display name, then item id.
pub fn compute_sort_rank(
    creative_order: usize,
    display_name: &str,
    item_id: &str,
) -> (usize, String) {
    let short_id = item_id.strip_prefix("minecraft:").unwrap_or(item_id);
    (
        creative_order,
        format!("{}\0{}", display_name.to_ascii_lowercase(), short_id),
    )
}

/// Slot ranges used by the mod (same numbering as Horizon's inventory screen).
pub const MAIN_INVENTORY: core::ops::Range<usize> = 9..36;
pub const CONTAINER_SLOTS: core::ops::Range<usize> = 300..500;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_order_wins_over_name() {
        let stone = compute_sort_rank(10, "Stone", "minecraft:stone");
        let apple = compute_sort_rank(250, "Apple", "minecraft:apple");
        assert!(stone < apple);
    }

    #[test]
    fn same_category_sorts_by_name_then_id() {
        let apple = compute_sort_rank(5, "Apple", "minecraft:apple");
        let bread = compute_sort_rank(5, "Bread", "minecraft:bread");
        assert!(apple < bread);
        let a = compute_sort_rank(5, "Planks", "minecraft:acacia_planks");
        let b = compute_sort_rank(5, "Planks", "minecraft:oak_planks");
        assert!(a < b);
    }

    #[test]
    fn toast_sets_timer_and_text() {
        push_toast("hello", COLOR_INFO);
        with_state(|st| {
            assert_eq!(st.toast_text, "hello");
            assert!(st.toast_timer > 0.0);
        });
    }
}
