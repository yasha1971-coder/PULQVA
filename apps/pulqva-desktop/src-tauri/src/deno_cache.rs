//! Private runtime workspace. Cleanup requires the child tree to have exited.
//! Identity checks detect replacement, but are not atomic hostile-race protection.
#![allow(dead_code)]
use same_file::Handle;
use std::{fs, path::{Path, PathBuf}, process::Command,
    sync::atomic::{AtomicU64, Ordering}};
use super::deno_environment::DenoEnvironment;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Directory { path: PathBuf, id: Handle }
impl Directory {
    fn open(path: PathBuf) -> Result<Self, &'static str> {
        let id = Handle::from_path(&path).map_err(|_| "directory-identity")?;
        let directory = Self { path, id };
        if !directory.owned() { return Err("directory-not-regular"); }
        Ok(directory)
    }
    fn owned(&self) -> bool { matches_identity(&self.path, &self.id, true) }
}
fn matches_identity(path: &Path, id: &Handle, directory: bool) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| !m.file_type().is_symlink()
        && if directory { m.is_dir() } else { m.is_file() })
        && Handle::from_path(path).is_ok_and(|h| &h == id)
}
fn mkdir(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
    builder.create(path)
}

pub(crate) struct DenoWorkspace {
    parent: Directory,
    root: Directory,
    children: Vec<Directory>,
}
impl DenoWorkspace {
    pub(crate) fn create(parent: &Path) -> Result<Self, &'static str> {
        if !parent.is_absolute() || fs::canonicalize(parent).map_err(|_| "parent-invalid")? != parent {
            return Err("parent-not-canonical");
        }
        let parent = Directory::open(parent.to_owned())?;
        for _ in 0..64 {
            let path = parent.path.join(format!(".deno-work-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
            match mkdir(&path) {
                Ok(()) => {},
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err("workspace-create"),
            }
            let root = Directory::open(path)?;
            let mut owned = Self { parent, root, children: Vec::new() };
            for name in ["home", "tmp", "cache"] {
                if !owned.owned() { return Err("workspace-ownership-lost"); }
                let path = owned.root.path.join(name);
                mkdir(&path).map_err(|_| "workspace-child-create")?;
                owned.children.push(Directory::open(path)?);
            }
            return Ok(owned);
        }
        Err("workspace-name-exhausted")
    }
    fn owned(&self) -> bool {
        self.parent.owned() && self.root.owned() && self.children.iter().all(Directory::owned)
    }
    pub(crate) fn apply(&self, command: &mut Command, system_root: Option<PathBuf>) -> Result<(), &'static str> {
        if !self.owned() { return Err("workspace-ownership-lost"); }
        DenoEnvironment::new(self.root.path.clone(), system_root)?.apply(command);
        Ok(())
    }
    // Explicit cleanup reports failure; Drop is best effort. Caller must retain
    // this owner until every runtime descendant has exited before invoking either.
    pub(crate) fn cleanup(&mut self) -> Result<(), &'static str> {
        if !self.owned() { return Err("workspace-ownership-lost"); }
        let mut entries = Vec::new();
        snapshot(&self.root.path, 0, &mut entries)?;
        for (path, id, directory) in entries {
            if !self.owned() || !matches_identity(&path, &id, directory) {
                return Err("workspace-entry-replaced");
            }
            // Keep the three tracked directories until all contents are removed.
            if self.children.iter().any(|d| d.path == path) { continue; }
            if directory { fs::remove_dir(path) } else { fs::remove_file(path) }
                .map_err(|_| "workspace-entry-cleanup")?;
        }
        while let Some(directory) = self.children.last() {
            if !self.parent.owned() || !self.root.owned() || !directory.owned() {
                return Err("workspace-ownership-lost");
            }
            fs::remove_dir(&directory.path).map_err(|_| "workspace-child-cleanup")?;
            self.children.pop();
        }
        if !self.parent.owned() || !self.root.owned() { return Err("workspace-ownership-lost"); }
        fs::remove_dir(&self.root.path).map_err(|_| "workspace-root-cleanup")
    }
}
impl Drop for DenoWorkspace { fn drop(&mut self) { let _ = self.cleanup(); } }

// Bounded postorder snapshot; never follows symlinks or special files. Unknown
// links stop cleanup before any deletion. Ordinary files inside this private
// workspace are runtime-owned cache/temp data, not user-selected output files.
fn snapshot(path: &Path, depth: usize, entries: &mut Vec<(PathBuf, Handle, bool)>) -> Result<(), &'static str> {
    if depth > 32 { return Err("workspace-depth-limit"); }
    for entry in fs::read_dir(path).map_err(|_| "workspace-read")? {
        if entries.len() >= 4096 { return Err("workspace-entry-limit"); }
        let path = entry.map_err(|_| "workspace-read")?.path();
        let m = fs::symlink_metadata(&path).map_err(|_| "workspace-stat")?;
        if m.file_type().is_symlink() || !(m.is_file() || m.is_dir()) { return Err("workspace-special-entry"); }
        let id = Handle::from_path(&path).map_err(|_| "workspace-entry-identity")?;
        if m.is_dir() { snapshot(&path, depth + 1, entries)?; }
        if entries.len() >= 4096 { return Err("workspace-entry-limit"); }
        entries.push((path, id, m.is_dir()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parent() -> PathBuf {
        let p = std::env::temp_dir().join(format!("pulqva-cache-test-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&p).unwrap(); fs::canonicalize(p).unwrap()
    }
    #[test]
    fn owned_cache_and_temp_are_removed_but_neighbor_is_preserved() {
        let parent = parent(); fs::write(parent.join("keep"), b"user").unwrap();
        let mut workspace = DenoWorkspace::create(&parent).unwrap();
        let root = workspace.root.path.clone();
        fs::create_dir(root.join("cache/analysis")).unwrap();
        fs::write(root.join("cache/analysis/data"), b"analysis-cache").unwrap();
        fs::write(root.join("tmp/partial"), b"partial").unwrap();
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt;
            for path in [&root, &root.join("cache"), &root.join("home"), &root.join("tmp")] {
                assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o777, 0o700);
            }
        }
        workspace.cleanup().unwrap(); drop(workspace);
        assert!(!root.exists()); assert_eq!(fs::read(parent.join("keep")).unwrap(), b"user");
        fs::remove_file(parent.join("keep")).unwrap(); fs::remove_dir(parent).unwrap();
    }
    #[test]
    fn error_return_drops_partial_runtime_data() {
        let parent = parent();
        let operation = || -> Result<(), &'static str> {
            let workspace = DenoWorkspace::create(&parent)?;
            fs::write(workspace.root.path.join("tmp/partial"), b"failed").unwrap();
            Err("injected-runtime-failure")
        };
        assert!(operation().is_err()); assert_eq!(fs::read_dir(&parent).unwrap().count(), 0);
        fs::remove_dir(parent).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn replacement_is_not_deleted_and_apply_fails() {
        let parent = parent(); let workspace = DenoWorkspace::create(&parent).unwrap();
        let root = workspace.root.path.clone(); let saved = parent.join("saved");
        fs::rename(&root, &saved).unwrap(); fs::create_dir(&root).unwrap();
        fs::write(root.join("foreign"), b"keep").unwrap();
        assert!(workspace.apply(&mut Command::new("unused"), None).is_err());
        drop(workspace); assert_eq!(fs::read(root.join("foreign")).unwrap(), b"keep");
        fs::remove_file(root.join("foreign")).unwrap(); fs::remove_dir(root).unwrap();
        for name in ["cache", "home", "tmp"] { fs::remove_dir(saved.join(name)).unwrap(); }
        fs::remove_dir(saved).unwrap(); fs::remove_dir(parent).unwrap();
    }
}
