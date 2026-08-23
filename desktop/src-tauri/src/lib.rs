mod commands;
mod local_database;
mod session;

use commands::launcher::LauncherFocusGuard;
use local_database::LocalDatabase;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(LocalDatabase::default())
        .manage(LauncherFocusGuard::default());
    #[cfg(target_os = "macos")]
    {
        builder = builder.manage(commands::launcher::PreviousApplication::default());
    }
    builder
        .invoke_handler(tauri::generate_handler![
            commands::database::initialize_local_database,
            commands::database::get_local_database_status,
            commands::database::count_local_prompts,
            commands::database::create_local_prompt,
            commands::database::import_downloaded_prompt,
            commands::database::list_local_prompts,
            commands::database::update_local_prompt,
            commands::database::delete_local_prompt,
            commands::database::list_local_categories,
            commands::database::create_local_category,
            commands::database::record_local_prompt_use,
            commands::database::create_local_collection,
            commands::database::list_local_collections,
            commands::database::add_prompt_to_local_collection,
            commands::database::list_local_collection_members,
            commands::database::get_local_setting,
            commands::database::set_local_setting,
            commands::database::export_local_library,
            commands::database::preview_local_import,
            commands::database::apply_local_import,
            commands::database::backup_local_library,
            commands::database::restore_local_library,
            commands::launcher::show_launcher,
            commands::launcher::hide_launcher,
            commands::launcher::hide_launcher_if_idle,
            commands::launcher::resize_launcher,
            commands::launcher::toggle_launcher,
            commands::paste::paste_to_active_app,
            commands::paste::capture_selected_text,
            commands::session::login_local_session,
            commands::session::logout_local_session,
            commands::session::refresh_local_session,
            commands::square::list_square_items,
            commands::square::download_square_item,
            commands::square::create_publication,
            commands::square::put_favorite,
            commands::square::delete_favorite,
            commands::square::list_favorites,
        ])
        .setup(|app| {
            let database = app.state::<LocalDatabase>();
            if let Ok(dir) = app.path().app_data_dir() {
                let _ = database.initialize(&dir);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running PromptArk");
}
