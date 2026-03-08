mod binary_manager;
mod commands;
mod config;
mod error;
mod model_manager;
mod queue;
mod source;
mod subtitle;
mod translate;
mod tts;
mod youtube;

use commands::{queue_cmd, settings_cmd, translate_cmd, tts_cmd, youtube_cmd};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "subflow=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .manage(queue::orchestrator::Orchestrator::new())
        .invoke_handler(tauri::generate_handler![
            youtube_cmd::download_subtitle,
            youtube_cmd::get_video_metadata,
            youtube_cmd::parse_subtitle_file,
            translate_cmd::translate_subtitle,
            tts_cmd::generate_tts,
            tts_cmd::list_tts_voices,
            queue_cmd::add_task,
            queue_cmd::cancel_task,
            queue_cmd::pause_task,
            queue_cmd::resume_task,
            queue_cmd::retry_task,
            queue_cmd::remove_task,
            queue_cmd::get_tasks,
            settings_cmd::get_settings,
            settings_cmd::save_settings,
            settings_cmd::save_api_key,
            settings_cmd::get_api_key_preview,
            settings_cmd::test_provider_connection,
            settings_cmd::ensure_directory,
            settings_cmd::open_folder,
            settings_cmd::setup_binaries,
            settings_cmd::get_binary_status,
            settings_cmd::download_nllb_model,
            settings_cmd::delete_nllb_model,
            settings_cmd::setup_ytdlp_env,
            settings_cmd::delete_ytdlp_env,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
