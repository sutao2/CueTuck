use super::{files, install, local, *};
use crate::skill_bundle::{Bundle, BundleFile};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{fs, path::Path};
pub fn create(data: &Path, defaults: &[Root], name: &str, body: &str) -> Result<PathBuf> {
    if !crate::skill_bundle::valid_name(name) || body.trim().is_empty() || body.len() > 1024 * 1024
    {
        return Err("名称须为小写英文、数字或短横线，正文不能为空且不超过 1 MiB".into());
    }
    let root = data.join("drafts");
    files::no_links(&root)?;
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let target = root.join(name);
    if target.exists() {
        return Err("同名 Skill 已存在，请更换名称".into());
    }
    let mut registry = files::load_registry(data)?;
    if !registry.roots.iter().any(|r| r.path == root) {
        local::register(
            &mut registry,
            defaults,
            Root {
                id: String::new(),
                name: "CueTuck 创作".into(),
                agent: "custom".into(),
                scope: "global".into(),
                path: root.clone(),
                readonly: false,
                custom: true,
                shared_with: vec![],
                status: String::new(),
            },
        )?;
        files::save_registry(data, &registry)?;
    }
    let stage = root.join(format!(".create-{}", id()));
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let result = (|| {
        fs::write(stage.join("SKILL.md"), body).map_err(|e| e.to_string())?;
        files::package(&stage)?;
        fs::rename(&stage, &target).map_err(|e| e.to_string())?;
        Ok(target)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(stage);
    }
    result
}
pub fn export(data: &Path, key: &str) -> Result<Bundle> {
    let prepared = install::prepared(data, key)?;
    let root = install::prepared_dir(data, key)?.join("package");
    let bundle = Bundle {
        name: prepared.folder_name,
        files: prepared
            .package
            .files
            .iter()
            .map(|file| {
                Ok(BundleFile {
                    path: file.path.clone(),
                    content: STANDARD.encode(files::read_bounded(
                        &root.join(files::relative(&file.path)?),
                        MAX_FILE,
                    )?),
                    executable: file.executable,
                })
            })
            .collect::<Result<_>>()?,
    };
    bundle.validate()?;
    Ok(bundle)
}
pub fn prepare(data: &Path, bundle: Bundle, expected_digest: &str) -> Result<Prepared> {
    let bytes = bundle.validate()?;
    if bundle.digest() != expected_digest {
        return Err("社区文件包校验失败，请重新加载".into());
    }
    fs::create_dir_all(data).map_err(|e| e.to_string())?;
    let stage = data.join(format!("import-{}", id()));
    files::no_links(&stage)?;
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let result = (|| {
        for (file, bytes) in bundle.files.iter().zip(bytes) {
            let path = stage.join(files::relative(&file.path)?);
            fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::write(&path, bytes).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    &path,
                    fs::Permissions::from_mode(if file.executable { 0o755 } else { 0o644 }),
                )
                .map_err(|e| e.to_string())?;
            }
        }
        if files::package(&stage)?.files.len() != bundle.files.len() {
            return Err("目标文件系统存在重名路径，不能完整安装此包".into());
        }
        install::store_prepared(data, bundle.name, &stage, None, None)
    })();
    let _ = fs::remove_dir_all(stage);
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_export_install_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let data = temp.path().join("data");
        let path = create(
            &data,
            &[],
            "my-skill",
            "---\nname: my-skill\ndescription: Test\n---\n# 内容",
        )
        .unwrap();
        assert!(create(&data, &[], "my-skill", "other").is_err());
        let mut registry = files::load_registry(&data).unwrap();
        assert_eq!(local::scan(&[], &registry).skills.len(), 1);
        let p = install::prepare_local(&data, &path).unwrap();
        let b = export(&data, &p.id).unwrap();
        let q = prepare(&data, b.clone(), &b.digest()).unwrap();
        assert_eq!(p.package.digest, q.package.digest);
        let target = Root {
            id: "target".into(),
            name: "Target".into(),
            agent: "custom".into(),
            scope: "global".into(),
            path: temp.path().join("target"),
            readonly: false,
            custom: true,
            shared_with: vec![],
            status: String::new(),
        };
        registry.roots.push(target.clone());
        let results = install::install_many(
            &data,
            &[],
            &mut registry,
            &q,
            &[install::Selection {
                root_id: "target".into(),
                expected_digest: None,
                replace: false,
            }],
        )
        .unwrap();
        assert_eq!(results[0].status, "success");
        assert_eq!(
            fs::read(target.path.join("my-skill/SKILL.md")).unwrap(),
            fs::read(path.join("SKILL.md")).unwrap()
        );
        assert!(prepare(&data, b, "wrong").is_err());
    }
}
