// Hook helper: native_mods
fn inventory_sorter_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::mod_inventory_sorter_glue::on_list(mods);
}
