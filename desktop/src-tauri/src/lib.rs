#[path = "../../../shared/search.rs"]
mod prompt_search;
#[path = "../../../shared/skill_bundle.rs"]
mod skill_bundle;
#[path = "../../../shared/translation.rs"]
mod prompt_translation;
mod commands;
mod skills;
mod api_config;
mod http;
mod local_database;
mod session;

use commands::launcher::LauncherFocusGuard;
use local_database::{get_setting_in_dir, LocalDatabase};
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(LocalDatabase::default())
        .manage(commands::skills::SkillsState::default())
        .manage(commands::updates::UpdateState::default())
        .manage(LauncherFocusGuard::default());
    #[cfg(target_os = "macos")]
    {
        builder = builder.manage(commands::launcher::PreviousApplication::default());
    }
    builder
        .invoke_handler(tauri::generate_handler![
            commands::skills::skills_command,
            commands::skills::skill_market_request,
            commands::skills::skills_choose_directory,
            commands::skills::skills_cancel,
            commands::ai::get_launcher_ai_config,
            commands::ai::save_launcher_ai_config,
            commands::ai::clear_launcher_ai_config,
            commands::ai::list_launcher_ai_models,
            commands::ai::optimize_launcher_prompt,
            commands::ai::get_prompt_translation,
            commands::ai::cache_downloaded_translation,
            commands::ai::translate_local_prompt,
            commands::database::initialize_local_database,
            commands::database::export_local_sync_changes,
            commands::database::apply_local_sync_changes,
            commands::database::get_local_database_status,
            commands::database::count_local_prompts,
            commands::database::create_local_prompt,
            commands::database::create_collection_prompt,
            commands::database::save_local_prompt_with_assets,
            commands::database::list_local_prompt_assets,
            commands::database::get_local_prompt_image,
            commands::database::get_local_prompt_thumbnail,
            commands::database::append_downloaded_assets,
            commands::database::export_local_prompt_asset,
            commands::media::upload_private_asset,
            commands::media::download_private_asset,
            commands::media::download_published_asset,
            commands::media::download_reference_image,
            commands::media::download_reference_images,
            commands::media::hash_private_asset,
            commands::mcp::mcp_connection_info,
            commands::database::import_downloaded_prompt,
            commands::database::upsert_synced_local_prompt,
            commands::database::list_local_prompts,
            commands::database::update_local_prompt,
            commands::database::move_local_prompt_category,
            commands::database::choose_library_backup,
            commands::database::delete_local_prompt,
            commands::database::list_deleted_local_items,
            commands::database::restore_deleted_local_item,
            commands::database::list_local_categories,
            commands::database::create_local_category,
            commands::database::delete_local_category,
            commands::database::record_local_prompt_use,
            commands::database::create_local_collection,
            commands::database::list_local_collections,
            commands::database::add_prompt_to_local_collection,
            commands::database::list_local_collection_members,
            commands::database::remove_prompt_from_local_collection,
            commands::database::update_local_collection,
            commands::database::delete_local_collection,
            commands::database::get_local_setting,
            commands::database::set_local_setting,
            commands::database::export_local_library,
            commands::database::preview_local_import,
            commands::database::apply_local_import,
            commands::database::backup_local_library,
            commands::database::set_auto_backup,
            commands::database::restore_local_library,
            commands::database::open_library_dir,
            commands::database::export_library_zip,
            commands::database::clear_local_prompt_use,
            commands::desktop::apply_launch_at_login,
            commands::desktop::apply_minimize_to_tray,
            commands::launcher::show_launcher,
            commands::launcher::hide_launcher,
            commands::launcher::resume_launcher,
            commands::launcher::copy_launcher_text,
            commands::launcher::hide_launcher_if_idle,
            commands::launcher::resize_launcher,
            commands::launcher::toggle_launcher,
            commands::launcher::open_new_prompt,
            commands::launcher::open_launcher_destination,
            commands::launcher::paste_recent_prompt,
            commands::paste::paste_to_active_app,
            commands::paste::capture_selected_text,
            commands::session::login_local_session,
            commands::session::start_oauth_session,
            commands::session::poll_oauth_session,
            commands::session::commit_oauth_session,
            commands::session::cancel_oauth_session,
            commands::session::list_oauth_providers,
            commands::session::identity_request,
            commands::session::logout_local_session,
            commands::session::refresh_local_session,
            commands::session::get_me,
            commands::session::put_me,
            commands::session::put_library_changes,
            commands::session::list_library_changes,
            commands::session::get_billing_status,
            commands::session::start_billing_checkout,
            commands::session::redeem_billing_code,
            commands::updates::check_for_updates,
            commands::updates::queue_update_install,
            commands::updates::install_downloaded_update,
            commands::square::list_square_items,
            commands::square_page::list_square_page,
            commands::square_page::cancel_square_page,
            commands::square::get_square_catalog,
            commands::square::square_translations,
            commands::square::square_reports,
            commands::square::get_square_content,
            commands::square::record_square_download,
            commands::square::create_publication,
            commands::square::list_my_publications,
            commands::square::put_favorite,
            commands::square::delete_favorite,
            commands::square::list_favorites,
        ])
        .setup(|app| {
            let database = app.state::<LocalDatabase>();
            if let Ok(dir) = app.path().app_data_dir() {
                if database.initialize(&dir).is_ok() {
                    local_database::start_auto_backup_worker(dir.clone());
                }
                crate::http::load_from_dir(&dir);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                let Ok(dir) = window.app_handle().path().app_data_dir() else {
                    return;
                };
                if get_setting_in_dir(&dir, "minimize_to_tray").ok().as_deref() == Some("1") {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running CueTuck");
}
