#![cfg(all(target_os = "macos", feature = "updater-artifact-test"))]
use tauri_plugin_updater::UpdaterExt;

// Explicit opt-in: downloads the published signed artifact, installs only into a TempDir,
// and never launches or restarts the user's application.
#[tokio::test]
#[ignore = "requires CUETUCK_UPDATE_MANIFEST and CUETUCK_UPDATE_VERSION"]
async fn published_macos_update_installs_in_isolated_app() {
    let endpoint = std::env::var("CUETUCK_UPDATE_MANIFEST").expect("manifest URL");
    let version = std::env::var("CUETUCK_UPDATE_VERSION").expect("expected version");
    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("CueTuck.app");
    let executable = destination.join("Contents/MacOS/cuetuck");
    std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
    std::fs::write(&executable, b"old isolated fixture").unwrap();
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert("updater".into(), config["plugins"]["updater"].clone());
    eprintln!("Building isolated mock application");
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context).unwrap();
    eprintln!("Mock application ready");
    let expected = version.clone();
    let updater = app.updater_builder()
        .pubkey(config["plugins"]["updater"]["pubkey"].as_str().unwrap())
        .endpoints(vec![endpoint.parse().unwrap()]).unwrap()
        .executable_path(&executable)
        .version_comparator(move |_, release| release.version.to_string() == expected)
        .timeout(std::time::Duration::from_secs(180))
        .build().unwrap();
    eprintln!("Checking published manifest");
    let update = updater.check().await.unwrap().expect("matching platform update");
    assert_eq!(update.version, version);
    eprintln!("Manifest selected; downloading signed artifact");
    let mut downloaded = 0;
    let bytes = update.download(|size, _| downloaded += size, || {}).await.expect("download and verify signature");
    assert_eq!(downloaded, bytes.len());
    assert!(downloaded > 1_000_000);
    eprintln!("Installing verified bytes into temporary application");
    update.install(bytes).expect("isolated app replacement");
    assert!(std::fs::metadata(&executable).unwrap().len() > 1_000_000);
    let plist = std::fs::read_to_string(destination.join("Contents/Info.plist")).unwrap();
    assert!(plist.contains(&version));
    assert!(std::process::Command::new("codesign").args(["--verify","--deep","--strict"]).arg(&destination).status().unwrap().success());
    println!("Verified signed download progress and isolated app installation: {version}, {downloaded} bytes");
}
