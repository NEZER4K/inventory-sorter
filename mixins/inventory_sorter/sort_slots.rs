// Hook helper: tweaks::Game::sort_slots (cancellable HEAD)
fn inventory_sorter_sort_slots(
    game: &mut crate::game::Game<'_>,
    slots: &[usize],
) -> core::ops::ControlFlow<()> {
    crate::game::mod_inventory_sorter_glue::on_sort_slots(game, slots)
}
