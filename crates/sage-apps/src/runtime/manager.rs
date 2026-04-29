use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppsHostState;
use crate::bridge::methods::system::RuntimeManagerRuntimesChangedEvent;
use crate::runtime::state::{
    get_runtime_by_app_id, list_runtimes, SageAppRuntimeRecord,
};
use crate::runtime::webview_locator::{find_sage_window, get_webview_in_sage_window};
use crate::runtime::SharedRuntimeRecordExt;

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTargetParams {
    pub app_id: String,
}

pub(crate) async fn emit_runtime_manager_runtimes_changed(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let Ok(runtimes) = list_runtimes(apps_state).await else {
        return;
    };

    let system_runtime_webview_labels = runtimes
        .iter()
        .filter_map(|runtime| {
            runtime.with_runtime(|record| {
                if record.internal() {
                    return None;
                }

                if !record.app().is_system() {
                    return None;
                }

                Some(record.webview_label().to_string())
            })
        })
        .collect::<Vec<_>>();

    let runtime_records = runtimes
        .iter()
        .map(|runtime| runtime.with_runtime(Clone::clone))
        .collect::<Vec<SageAppRuntimeRecord>>();

    let event = RuntimeManagerRuntimesChangedEvent::new(
        "sage-system-bridge".to_string(),
        runtime_records,
    );

    let Some(sage_window) = find_sage_window(app) else {
        return;
    };

    for system_webview_label in system_runtime_webview_labels {
        if let Some(webview) = sage_window.get_webview(&system_webview_label) {
            let _ = webview.emit("sage-system-bridge:event", event.clone());
        }
    }
}

pub(crate) async fn focus_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SageAppRuntimeRecord, String> {
    let runtime = get_runtime_by_app_id(apps_state, app_id).await?;

    let webview_label = runtime.with_runtime(|record| record.webview_label().to_string());

    let webview = get_webview_in_sage_window(app, &webview_label)?;

    webview
        .show()
        .map_err(|err| format!("failed to show webview: {err}"))?;

    webview
        .set_focus()
        .map_err(|err| format!("failed to focus webview: {err}"))?;

    let snapshot = runtime.with_runtime(|record| {
        let mut record = record.clone();
        record.mark_visible();
        record
    });

    {
        let mut record = runtime.write();
        record.mark_visible();
    }

    emit_runtime_manager_runtimes_changed(app, apps_state).await;

    Ok(snapshot)
}

pub(crate) async fn hide_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SageAppRuntimeRecord, String> {
    let runtime = get_runtime_by_app_id(apps_state, app_id).await?;

    let webview_label = runtime.with_runtime(|record| record.webview_label().to_string());

    let webview = get_webview_in_sage_window(app, &webview_label)?;

    webview
        .hide()
        .map_err(|err| format!("failed to hide webview: {err}"))?;

    let snapshot = runtime.with_runtime(|record| {
        let mut record = record.clone();
        record.mark_hidden();
        record
    });

    {
        let mut record = runtime.write();
        record.mark_hidden();
    }

    emit_runtime_manager_runtimes_changed(app, apps_state).await;

    Ok(snapshot)
}
