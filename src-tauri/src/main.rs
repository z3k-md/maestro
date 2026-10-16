#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        log::error!("Panic occurred: {}", panic_info);
        default_hook(panic_info);
    }));

    maestro_lib::run();
}
