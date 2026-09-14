use super::*;
use std::{fs, path::Path};
fn skill(path: &Path, body: &str) {
    fs::create_dir_all(path.join("scripts")).unwrap();
    fs::write(
        path.join("SKILL.md"),
        format!("---\nname: sample\ndescription: >\n  A multiline\n  description\n---\n{body}"),
    )
    .unwrap();
    fs::write(path.join("scripts/run.sh"), "echo never-run").unwrap();
}
fn root(path: &Path) -> Root {
    Root {
        id: "test".into(),
        name: "Test".into(),
        agent: "custom".into(),
        scope: "global".into(),
        path: path.into(),
        readonly: false,
        custom: true,
        shared_with: vec![],
        status: String::new(),
    }
}
#[test]
fn opens_scanned_skill_details_with_canonical_unicode_paths() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("用户 Skills");
    let path = dir.join("插件 缓存/sample");
    skill(&path, "Detail content");
    for readonly in [false, true] {
        let mut r = root(&dir);
        r.readonly = readonly;
        let roots = vec![r];
        let registry = Registry::default();
        let scan = local::scan(&roots, &registry);
        assert_eq!(scan.skills.len(), 1, "{:?}", scan.warnings);
        let detail = local::detail(&roots, &registry, &scan.skills[0].path).unwrap();
        assert!(detail.body.contains("Detail content"));
        assert_eq!(detail.files.len(), 2);
        let canonical = path.canonicalize().unwrap();
        assert_eq!(files::package(&canonical).unwrap().digest, detail.digest);
    }
}
#[test]
fn parses_yaml_and_copies_complete_package_without_execution() {
    let t = tempfile::tempdir().unwrap();
    let src = t.path().join("source");
    skill(&src, "Content");
    let p = files::package(&src).unwrap();
    assert_eq!(p.description, "A multiline description");
    assert_eq!(p.files.len(), 2);
    let target = t.path().join("target");
    files::copy_package(&src, &target, &p).unwrap();
    assert_eq!(files::package(&target).unwrap().digest, p.digest);
    fs::write(src.join("SKILL.md"), "changed").unwrap();
    assert!(files::copy_package(&src, &t.path().join("other"), &p).is_err());
}
#[test]
fn rejects_unsafe_paths_and_oversize_files() {
    for path in [
        "../outside",
        "/tmp/test",
        "a/../b",
        "C:/bad",
        "a\\b",
        "a//b",
        "CON.txt",
        "a.",
    ] {
        assert!(files::relative(path).is_err(), "{path}");
    }
    let t = tempfile::tempdir().unwrap();
    skill(t.path(), "body");
    let f = fs::File::create(t.path().join("large")).unwrap();
    f.set_len(MAX_FILE + 1).unwrap();
    assert!(files::package(t.path()).is_err());
}
#[test]
fn discovers_nested_same_names_separately_and_persists_custom_roots() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path().join("root");
    skill(&r.join("one"), "first");
    skill(&r.join("nested/two"), "second");
    let mut registry = Registry::default();
    let mut custom = root(&r);
    custom.id.clear();
    local::register(&mut registry, &[], custom).unwrap();
    let dir = t.path().join("data");
    files::save_registry(&dir, &registry).unwrap();
    let registry = files::load_registry(&dir).unwrap();
    let scan = local::scan(&[], &registry);
    assert_eq!(scan.skills.len(), 2);
    assert_ne!(scan.skills[0].key, scan.skills[1].key);
    assert_eq!(scan.skills[0].status, "外部安装");
}
#[test]
fn supports_environment_overrides_and_explicit_project_paths() {
    let t = tempfile::tempdir().unwrap();
    let override_dir = t.path().join("config");
    let roots = local::default_roots(t.path(), |key| {
        if key == "CODEX_HOME" {
            Some(override_dir.to_string_lossy().into_owned())
        } else {
            None
        }
    });
    assert_eq!(
        roots.iter().find(|r| r.id == "codex-legacy").unwrap().path,
        override_dir.join("skills")
    );
    let project = local::project_roots(t.path()).unwrap();
    assert_eq!(project.len(), 5);
    assert!(project.iter().all(|r| r.scope == "project"));
    assert!(!project[0].path.exists());
}
#[cfg(unix)]
#[test]
fn marks_links_readonly_and_rejects_embedded_link_escape() {
    use std::os::unix::fs::symlink;
    let t = tempfile::tempdir().unwrap();
    let outside = t.path().join("outside");
    skill(&outside, "body");
    let r = t.path().join("root");
    fs::create_dir(&r).unwrap();
    symlink(&outside, r.join("linked")).unwrap();
    symlink(&r, r.join("loop")).unwrap();
    let registry = Registry {
        roots: vec![root(&r)],
        ..Default::default()
    };
    let scan = local::scan(&[], &registry);
    assert_eq!(scan.skills.len(), 1);
    assert!(scan.skills[0].readonly);
    assert!(!scan.warnings.is_empty());
    symlink("/etc/passwd", outside.join("escape")).unwrap();
    assert!(files::package(&outside).is_err());
}
#[test]
fn does_not_overwrite_corrupt_registry() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("registry.json"), "bad").unwrap();
    assert!(files::load_registry(t.path()).is_err());
    assert_eq!(
        fs::read_to_string(t.path().join("registry.json")).unwrap(),
        "bad"
    );
}

fn setup() -> (tempfile::TempDir, PathBuf, Vec<Root>, Registry, Prepared) {
    let t = tempfile::tempdir().unwrap();
    let data = t.path().join("data");
    let source = t.path().join("sample");
    skill(&source, "first");
    let roots = vec![root(&t.path().join("destination"))];
    let p = install::prepare_local(&data, &source).unwrap();
    (t, data, roots, Registry::default(), p)
}
fn selection(root: &Root, digest: Option<String>, replace: bool) -> install::Selection {
    install::Selection {
        root_id: root.id.clone(),
        expected_digest: digest,
        replace,
    }
}
#[test]
fn installs_complete_package_and_refuses_unconfirmed_or_changed_conflicts() {
    let (t, data, roots, mut registry, p) = setup();
    let target = install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    assert!(target.join("scripts/run.sh").is_file());
    assert_eq!(files::package(&target).unwrap().digest, p.package.digest);
    let second = t.path().join("other/sample");
    skill(&second, "second");
    let next = install::prepare_local(&data, &second).unwrap();
    let plan = install::plan(&roots, &registry, &next, &roots[0].id).unwrap();
    assert_eq!(plan.status, "conflict");
    assert!(install::install_one(
        &data,
        &roots,
        &mut registry,
        &next,
        &selection(&roots[0], plan.current_digest.clone(), false)
    )
    .is_err());
    fs::write(target.join("SKILL.md"), "locally changed").unwrap();
    assert!(install::install_one(
        &data,
        &roots,
        &mut registry,
        &next,
        &selection(&roots[0], plan.current_digest, true)
    )
    .is_err());
    let plan = install::plan(&roots, &registry, &next, &roots[0].id).unwrap();
    assert_eq!(plan.status, "modified");
    install::install_one(
        &data,
        &roots,
        &mut registry,
        &next,
        &selection(&roots[0], plan.current_digest, true),
    )
    .unwrap();
    assert_eq!(registry.backups.len(), 1);
    assert_eq!(
        fs::read_to_string(registry.backups[0].path.join("SKILL.md")).unwrap(),
        "locally changed"
    );
}
#[test]
fn remove_restore_survives_restart_and_refuses_collision() {
    let (_t, data, roots, mut registry, p) = setup();
    let target = install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    let backup = install::remove(
        &data,
        &roots,
        &mut registry,
        &target,
        &roots[0].id,
        &p.package.digest,
    )
    .unwrap();
    assert!(!target.exists());
    let mut registry = files::load_registry(&data).unwrap();
    install::restore(&data, &roots, &mut registry, &backup.id).unwrap();
    assert_eq!(files::package(&target).unwrap().digest, p.package.digest);
    assert!(install::restore(&data, &roots, &mut registry, &backup.id).is_err());
    assert!(backup.path.is_dir());
}
#[test]
fn multiple_targets_report_partial_failure_and_protect_readonly_roots() {
    let (t, data, mut roots, mut registry, p) = setup();
    let mut blocked = root(&t.path().join("protected"));
    blocked.id = "protected".into();
    blocked.readonly = true;
    roots.push(blocked);
    let results = install::install_many(
        &data,
        &roots,
        &mut registry,
        &p,
        &[
            selection(&roots[0], None, false),
            selection(&roots[1], None, false),
        ],
    )
    .unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].status, "success");
    assert_eq!(results[1].status, "failed");
    assert!(!roots[1].path.exists());
}
#[test]
fn same_repo_version_deduplicates_but_different_sources_do_not() {
    let (_t, data, roots, mut registry, mut p) = setup();
    p.source = Some(Source {
        repo: "owner/repo".into(),
        reference: "main".into(),
        commit: "a".repeat(40),
        directory: "sample".into(),
        license: "MIT".into(),
    });
    install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    let plan = install::plan(&roots, &registry, &p, &roots[0].id).unwrap();
    assert_eq!(plan.status, "same");
    install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], plan.current_digest, false),
    )
    .unwrap();
    assert!(registry.backups.is_empty());
    p.source.as_mut().unwrap().repo = "other/repo".into();
    assert_eq!(
        install::plan(&roots, &registry, &p, &roots[0].id)
            .unwrap()
            .status,
        "conflict"
    );
}
#[test]
fn recovers_interrupted_rename_and_rejects_escape_paths() {
    let (_t, data, roots, mut registry, p) = setup();
    let target = install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    let rollback = roots[0].path.join(format!(".cuetuck-rollback-{}", id()));
    let stage = roots[0].path.join(format!(".cuetuck-stage-{}", id()));
    fs::rename(&target, &rollback).unwrap();
    registry.pending = Some(Pending {
        target: target.clone(),
        stage,
        rollback: rollback.clone(),
        installation: registry.installations.get(&target).cloned(),
        remove: false,
        expected_digest: Some(p.package.digest.clone()),
    });
    files::save_registry(&data, &registry).unwrap();
    let mut loaded = files::load_registry(&data).unwrap();
    install::recover(&data, &mut loaded).unwrap();
    assert!(target.exists());
    assert!(!rollback.exists());
    assert!(loaded.pending.is_none());
    assert!(local::detail(&roots, &loaded, &roots[0].path.join("../../outside")).is_err());
    assert!(install::prepared_dir(&data, "../escape").is_err());
}
#[test]
fn rollback_keeps_original_when_staging_is_missing() {
    let (_t, data, roots, mut registry, p) = setup();
    let target = install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    let stage = roots[0].path.join(format!(".cuetuck-stage-{}", id()));
    let rollback = roots[0].path.join(format!(".cuetuck-rollback-{}", id()));
    fs::rename(&target, &rollback).unwrap();
    let mut new = registry.installations.get(&target).unwrap().clone();
    new.digest = "different".into();
    registry.pending = Some(Pending {
        target: target.clone(),
        stage,
        rollback,
        installation: Some(new),
        remove: false,
        expected_digest: Some(p.package.digest.clone()),
    });
    files::save_registry(&data, &registry).unwrap();
    install::recover(&data, &mut registry).unwrap();
    assert_eq!(files::package(&target).unwrap().digest, p.package.digest);
    assert_eq!(registry.installations[&target].digest, p.package.digest);
}
#[cfg(unix)]
#[test]
fn preserves_executable_files_and_rejects_special_files() {
    use std::os::unix::fs::PermissionsExt;
    let (t, data, roots, mut registry, _) = setup();
    let source = t.path().join("sample");
    fs::set_permissions(
        source.join("scripts/run.sh"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let p = install::prepare_local(&data, &source).unwrap();
    let target = install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    assert_ne!(
        fs::metadata(target.join("scripts/run.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o111,
        0
    );
    let socket = std::os::unix::net::UnixListener::bind(source.join("socket")).unwrap();
    assert!(files::package(&source).is_err());
    drop(socket);
}
#[test]
fn parses_github_source_inputs_without_interpreting_branch_segments() {
    assert_eq!(
        remote::parse("https://github.com/owner/repo/tree/feature/skills/one")
            .unwrap()
            .tail,
        vec!["feature", "skills", "one"]
    );
    for input in [
        "https://evil.test/o/r",
        "https://github.com/o/r?token=private",
        "../repo",
        "file:///tmp/skill",
        "https://github.com/o/r/tree/main/../bad",
        "o/r/pulls/1",
    ] {
        assert!(remote::parse(input).is_err(), "{input}");
    }
}

struct Server {
    url: String,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Server {
    fn new(route: impl Fn(&str) -> (u16, String) + Send + 'static) -> Self {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stopped = stop.clone();
        let thread = std::thread::spawn(move || {
            while !stopped.load(std::sync::atomic::Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                            .unwrap();
                        let mut data = Vec::new();
                        let mut chunk = [0; 1024];
                        while data.len() < 8192 && !data.windows(4).any(|w| w == b"\r\n\r\n") {
                            let count = stream.read(&mut chunk).unwrap_or(0);
                            if count == 0 {
                                break;
                            }
                            data.extend_from_slice(&chunk[..count]);
                        }
                        let request = String::from_utf8_lossy(&data);
                        let path = request.split_whitespace().nth(1).unwrap_or("");
                        let (status, body) = route(path);
                        let _=write!(stream,"HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}",body.len());
                    }
                    Err(_) => std::thread::sleep(std::time::Duration::from_millis(2)),
                }
            }
        });
        Self {
            url,
            stop,
            thread: Some(thread),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            t.join().unwrap();
        }
    }
}
fn remote_route(path: &str) -> (u16, String) {
    let sha = "a".repeat(40);
    let body = "---\nname: remote\ndescription: actual description\n---\n# Real body";
    if path == "/repos/o/r" {
        return (
            200,
            r#"{"full_name":"o/r","default_branch":"main","license":{"spdx_id":"MIT"}}"#.into(),
        );
    }
    if path == "/repos/o/r/commits/main" || path == "/repos/o/r/commits/feature%2Fskills" {
        return (200, format!("{{\"sha\":\"{sha}\"}}"));
    }
    if path.starts_with("/repos/o/r/git/trees/") {
        return(200,serde_json::json!({"truncated":false,"tree":[{"path":"one/SKILL.md","mode":"100644","type":"blob","size":body.len()},{"path":"one/scripts/run.sh","mode":"100755","type":"blob","size":4}]}).to_string());
    }
    if path == format!("/o/r/{sha}/one/SKILL.md") {
        return (200, body.into());
    }
    if path == format!("/o/r/{sha}/one/scripts/run.sh") {
        return (200, "echo".into());
    }
    (404, "{}".into())
}
#[tokio::test]
async fn resolves_slash_branch_downloads_fixed_commit_and_installs_all_files() {
    use std::sync::atomic::AtomicBool;
    let server = Server::new(remote_route);
    let github = remote::Github::test(&server.url);
    let cancel = AtomicBool::new(false);
    let catalog = github
        .catalog("https://github.com/o/r/tree/feature/skills/one", &cancel)
        .await
        .unwrap();
    assert_eq!(catalog.reference, "feature/skills");
    assert_eq!(catalog.entries.len(), 1);
    let t = tempfile::tempdir().unwrap();
    let source = Source {
        repo: catalog.repo,
        reference: catalog.reference,
        commit: catalog.commit,
        directory: catalog.entries[0].directory.clone(),
        license: catalog.license,
    };
    let p = github
        .prepare(t.path(), source, &cancel, |_, _, _| {})
        .await
        .unwrap();
    assert_eq!(p.package.files.len(), 2);
    assert_eq!(p.package.description, "actual description");
    let roots = vec![root(&t.path().join("target"))];
    let mut registry = Registry::default();
    let target = install::install_one(
        t.path(),
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(target.join("scripts/run.sh")).unwrap(),
        "echo"
    );
    let update = github
        .update(
            t.path(),
            registry.installations.get(&target).unwrap(),
            &cancel,
            |_, _, _| {},
        )
        .await
        .unwrap();
    assert_eq!(update.package.digest, p.package.digest);
}
#[tokio::test]
async fn remote_failures_are_explicit_and_cannot_write_targets() {
    use std::sync::atomic::AtomicBool;
    let cancel = AtomicBool::new(false);
    for response in ["rate", "truncated", "symlink", "size", "missing"] {
        let server = Server::new(move |path| {
            if response == "rate" {
                return (403, "{}".into());
            }
            if path.contains("/git/trees/") {
                match response{
            "truncated"=>return(200,r#"{"truncated":true,"tree":[]}"#.into()),
            "symlink"=>return(200,r#"{"tree":[{"path":"one/SKILL.md","mode":"120000","type":"blob","size":4}]}"#.into()),
            "size"=>return(200,serde_json::json!({"tree":[{"path":"one/SKILL.md","mode":"100644","type":"blob","size":MAX_FILE+1}]}).to_string()),_=>{}}
            }
            if response == "missing" && path.ends_with("SKILL.md") {
                return (404, "{}".into());
            }
            remote_route(path)
        });
        let github = remote::Github::test(&server.url);
        let t = tempfile::tempdir().unwrap();
        let source = Source {
            repo: "o/r".into(),
            reference: "main".into(),
            commit: "a".repeat(40),
            directory: "one".into(),
            license: "未知".into(),
        };
        assert!(
            github
                .prepare(t.path(), source, &cancel, |_, _, _| {})
                .await
                .is_err(),
            "{response}"
        );
        assert!(!t.path().join("staging").exists());
    }
    let server = Server::new(remote_route);
    let github = remote::Github::test(&server.url);
    assert!(github
        .catalog("o/r", &AtomicBool::new(true))
        .await
        .unwrap_err()
        .contains("取消"));
}
#[tokio::test]
#[ignore = "explicit real public network check; only writes an isolated temporary directory"]
async fn real_public_skill_download_to_isolated_target() {
    let github = remote::Github::new().unwrap();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let catalog = github.catalog("anthropics/skills", &cancel).await.unwrap();
    let entry = catalog
        .entries
        .iter()
        .find(|e| e.name == "algorithmic-art")
        .or(catalog.entries.first())
        .unwrap();
    let t = tempfile::tempdir().unwrap();
    let p = github
        .prepare(
            t.path(),
            Source {
                repo: catalog.repo,
                reference: catalog.reference,
                commit: catalog.commit,
                directory: entry.directory.clone(),
                license: catalog.license,
            },
            &cancel,
            |_, _, _| {},
        )
        .await
        .unwrap();
    assert!(!p.package.files.is_empty());
    let roots = vec![root(&t.path().join("isolated-target"))];
    let mut registry = Registry::default();
    let path = install::install_one(
        t.path(),
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    assert_eq!(files::package(&path).unwrap().digest, p.package.digest);
    eprintln!(
        "verified public repo {} directory {} commit {} files {} bytes {}",
        p.source.as_ref().unwrap().repo,
        p.source.as_ref().unwrap().directory,
        p.source.as_ref().unwrap().commit,
        p.package.files.len(),
        p.package.bytes
    );
}

#[tokio::test]
#[ignore = "explicit public source reachability verification"]
async fn real_public_catalogs_are_accessible() {
    let github = remote::Github::new().unwrap();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let sources: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../src/data/skill-sources.json")).unwrap();
    for source in sources {
        let repo = source["value"].as_str().unwrap();
        let catalog = github.catalog(repo, &cancel).await.unwrap();
        assert!(!catalog.entries.is_empty());
        eprintln!(
            "catalog {} commit {} entries {} license {}",
            repo,
            catalog.commit,
            catalog.entries.len(),
            catalog.license
        );
    }
}

#[test]
fn cannot_write_protected_descendants_through_a_writable_parent() {
    let (t, data, mut roots, mut registry, mut p) = setup();
    let mut protected = root(&roots[0].path.join(".system"));
    protected.id = "system".into();
    protected.readonly = true;
    roots.push(protected);
    p.folder_name = ".system".into();
    assert!(install::plan(&roots, &registry, &p, &roots[0].id).is_err());
    let protected_skill = roots[1].path.join("existing");
    skill(&protected_skill, "protected");
    let digest = files::package(&protected_skill).unwrap().digest;
    assert!(install::remove(
        &data,
        &roots,
        &mut registry,
        &protected_skill,
        &roots[0].id,
        &digest
    )
    .is_err());
    assert!(protected_skill.exists());
    drop(t);
}

#[test]
fn handles_encoded_github_paths_and_rejects_encoded_traversal() {
    assert_eq!(
        remote::parse("https://github.com/o/r/tree/main/%E4%B8%AD%E6%96%87%20Skill")
            .unwrap()
            .tail,
        vec!["main", "中文 Skill"]
    );
    assert!(remote::parse("https://github.com/o/r/tree/main/%2e%2e/outside").is_err());
    assert!(remote::parse("o/.git").is_err());
}
#[test]
fn copying_a_managed_skill_retains_only_verified_upstream_identity() {
    let (_t, data, roots, mut registry, mut p) = setup();
    p.source = Some(Source {
        repo: "o/r".into(),
        reference: "main".into(),
        commit: "a".repeat(40),
        directory: "sample".into(),
        license: "MIT".into(),
    });
    let target = install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    assert_eq!(
        install::prepare_local(&data, &target).unwrap().source,
        p.source
    );
    fs::write(target.join("SKILL.md"), "locally changed").unwrap();
    assert!(install::prepare_local(&data, &target)
        .unwrap()
        .source
        .is_none());
}
#[test]
fn allows_editing_registered_project_labels_before_first_install() {
    let t = tempfile::tempdir().unwrap();
    let mut registry = Registry {
        roots: local::project_roots(t.path()).unwrap(),
        ..Registry::default()
    };
    let mut root = registry.roots[0].clone();
    root.name = "Renamed project".into();
    local::register(&mut registry, &[], root.clone()).unwrap();
    assert!(!root.path.exists());
    assert_eq!(registry.roots.last().unwrap().name, "Renamed project");
}
#[tokio::test]
async fn descriptions_are_real_and_partial_errors_stay_visible() {
    let server = Server::new(remote_route);
    let github = remote::Github::test(&server.url);
    let source = Source {
        repo: "o/r".into(),
        reference: "main".into(),
        commit: "a".repeat(40),
        directory: String::new(),
        license: "未知".into(),
    };
    let entries = vec!["one", "missing"]
        .into_iter()
        .map(|directory| remote::Entry {
            directory: directory.into(),
            name: directory.into(),
            description: String::new(),
            error: String::new(),
            loaded: false,
        })
        .collect();
    let result = github
        .descriptions(&source, entries, &std::sync::atomic::AtomicBool::new(false))
        .await
        .unwrap();
    assert_eq!(result[0].description, "actual description");
    assert!(result[0].loaded);
    assert!(!result[1].error.is_empty());
    assert!(!result[1].loaded);
}

#[test]
fn yaml_alias_expansion_and_excessive_nesting_are_not_evaluated() {
    let (_, description, _, warnings) = files::metadata(
        "---\na: &a [x,x]\nb: &b [*a,*a]\nname: bomb\ndescription: *b\n---\nbody",
        "fallback",
    );
    assert!(description.is_empty());
    assert!(!warnings.is_empty());
    let body = format!("---\na: {}1{}\n---", "[".repeat(50), "]".repeat(50));
    assert!(!files::metadata(&body, "fallback").3.is_empty());
}
#[test]
fn preview_cache_is_bounded_and_durable_backups_are_not_pruned() {
    let (_t, data, roots, mut registry, p) = setup();
    let target = install::install_one(
        &data,
        &roots,
        &mut registry,
        &p,
        &selection(&roots[0], None, false),
    )
    .unwrap();
    let b = install::remove(
        &data,
        &roots,
        &mut registry,
        &target,
        &roots[0].id,
        &p.package.digest,
    )
    .unwrap();
    for _ in 0..10 {
        install::prepare_local(&data, p.local_path.as_ref().unwrap()).unwrap();
    }
    assert!(fs::read_dir(data.join("staging")).unwrap().count() <= 8);
    assert!(b.path.join("SKILL.md").is_file());
}

#[tokio::test]
#[ignore = "explicit expanded public source download check; isolated temporary files only"]
async fn real_public_expanded_source_packages() {
    let github = remote::Github::new().unwrap();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let sources: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../src/data/skill-sources.json")).unwrap();
    for source in sources.into_iter().skip(4) {
        let repo = source["value"].as_str().unwrap();
        let catalog = github.catalog(repo, &cancel).await.unwrap();
        let entry = catalog.entries.first().expect("source must contain Skills");
        let t = tempfile::tempdir().unwrap();
        let p = github
            .prepare(
                t.path(),
                Source {
                    repo: catalog.repo.clone(),
                    reference: catalog.reference,
                    commit: catalog.commit.clone(),
                    directory: entry.directory.clone(),
                    license: catalog.license.clone(),
                },
                &cancel,
                |_, _, _| {},
            )
            .await
            .unwrap_or_else(|e| panic!("{repo}: {e}"));
        assert!(p.package.files.iter().any(|f| f.path == "SKILL.md"));
        assert_eq!(
            files::package(
                &install::prepared_dir(t.path(), &p.id)
                    .unwrap()
                    .join("package")
            )
            .unwrap()
            .digest,
            p.package.digest
        );
        eprintln!(
            "verified {} entries {} license {} commit {} sample {} files {} bytes {}",
            repo,
            catalog.entries.len(),
            catalog.license,
            catalog.commit,
            entry.directory,
            p.package.files.len(),
            p.package.bytes
        );
    }
}
