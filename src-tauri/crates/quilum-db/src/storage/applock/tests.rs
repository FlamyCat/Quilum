use crate::{Storage, storage::model::app_identifier::AppIdentifier};
use std::path::PathBuf;

#[tokio::test]
async fn blocked_apps_get_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert!(
        apps.is_empty(),
        "Should return empty list when no apps are blocked"
    );
}

#[tokio::test]
async fn blocked_apps_add_and_get() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let app = storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add blocked app");

    assert_eq!(app.display_name, "Firefox");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 1, "Should have 1 blocked app");
    assert_eq!(apps[0].display_name, "Firefox");
}

#[tokio::test]
async fn blocked_apps_add_multiple() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add Firefox");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/code")),
            "VS Code",
        )
        .await
        .expect("Failed to add VS Code");

    storage
        .upsert_blocked_app(
            AppIdentifier::BundleId("com.spotify.client".to_string()),
            "Spotify",
        )
        .await
        .expect("Failed to add Spotify");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 3, "Should have 3 blocked apps");
}

#[tokio::test]
async fn blocked_apps_upsert_updates_existing() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add Firefox");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox Updated",
        )
        .await
        .expect("Failed to update Firefox");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 1, "Should still have 1 app (not 2)");
    assert_eq!(
        apps[0].display_name, "Firefox Updated",
        "Name should be updated"
    );
}

#[tokio::test]
async fn blocked_apps_delete_all() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add Firefox");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/code")),
            "VS Code",
        )
        .await
        .expect("Failed to add VS Code");

    storage
        .delete_all_blocked_apps()
        .await
        .expect("Failed to delete all blocked apps");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert!(apps.is_empty(), "Should be empty after delete all");
}

#[tokio::test]
async fn blocked_apps_path_and_bundle_id() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox (path)",
        )
        .await
        .expect("Failed to add path-based app");

    storage
        .upsert_blocked_app(
            AppIdentifier::BundleId("com.spotify.client".to_string()),
            "Spotify (bundle)",
        )
        .await
        .expect("Failed to add bundle ID app");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(
        apps.len(),
        2,
        "Should have 2 apps with different identifier types"
    );
}

#[tokio::test]
async fn blocked_apps_old_add_method() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let app = storage
        .add_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add blocked app");

    assert_eq!(app.display_name, "Firefox");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 1, "Should have 1 blocked app");
}
