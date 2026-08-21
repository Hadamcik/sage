use tauri::{AppHandle, State, command};

use crate::{
    AppsHostState, RecoverableAppUpdateOutcome, Result, SageAppUrlPreview, SageAppView,
    apply_app_update_inner, apply_recoverable_app_update_inner, check_app_update_inner,
};

#[command]
#[specta::specta]
pub async fn apps_check_app_update(
    apps_state: State<'_, AppsHostState>,
    app_handle: AppHandle,
    app_id: String,
) -> Result<Option<SageAppUrlPreview>> {
    check_app_update_inner(&app_handle, &apps_state, &app_id).await
}

#[command]
#[specta::specta]
pub async fn apps_apply_app_update(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    app_id: String,
) -> Result<SageAppView> {
    apply_app_update_inner(&app_handle, &apps_state, &app_id, None, None).await
}

#[command]
#[specta::specta]
pub async fn apps_recover_app_update(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    app_id: String,
) -> Result<()> {
    match apply_recoverable_app_update_inner(&app_handle, &apps_state, &app_id, None, None, true)
        .await?
    {
        RecoverableAppUpdateOutcome::Applied(_) | RecoverableAppUpdateOutcome::ReviewOpened => {
            Ok(())
        }
        RecoverableAppUpdateOutcome::NotReady => {
            Err(std::io::Error::other(format!("app {app_id} has no recovery update ready")).into())
        }
    }
}
