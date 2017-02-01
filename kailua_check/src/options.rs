use std::str;
use std::path::{Path, PathBuf};

use kailua_syntax::Chunk;
use diag::CheckResult;

pub trait Options {
    fn set_package_path(&mut self, _path: &[u8]) -> CheckResult<()> { Ok(()) }
    fn set_package_cpath(&mut self, _path: &[u8]) -> CheckResult<()> { Ok(()) }

    fn require_chunk(&mut self, _path: &[u8]) -> CheckResult<Chunk> {
        Err("not implemented".into())
    }
}

pub trait FsSource {
    fn chunk_from_path(&self, resolved_path: &Path) -> CheckResult<Option<Chunk>>;
}

pub struct FsOptions<S> {
    source: S,
    root: PathBuf,
    package_path: Vec<String>,
    package_cpath: Vec<String>,
}

impl<S: FsSource> FsOptions<S> {
    pub fn new(source: S, root: PathBuf) -> FsOptions<S> {
        FsOptions {
            source: source,
            root: root,

            // by default, local files only
            package_path: vec!["?.lua".into()],
            package_cpath: vec![],
        }
    }

    fn search_file(&self, path: &str, search_paths: &[String],
                   suffix: &str) -> CheckResult<Option<Chunk>> {
        for template in search_paths {
            let path = template.replace('?', &path) + suffix;
            let path = self.root.join(path);
            debug!("trying to load {:?}", path);

            if let Some(chunk) = self.source.chunk_from_path(&path)? {
                return Ok(Some(chunk));
            }
        }
        Ok(None)
    }
}

impl<S: FsSource> Options for FsOptions<S> {
    fn set_package_path(&mut self, path: &[u8]) -> Result<(), String> {
        let path = str::from_utf8(path).map_err(|e| e.to_string())?;
        self.package_path = path.split(";").map(|s| s.to_owned()).collect();
        Ok(())
    }

    fn set_package_cpath(&mut self, path: &[u8]) -> Result<(), String> {
        let path = str::from_utf8(path).map_err(|e| e.to_string())?;
        self.package_cpath = path.split(";").map(|s| s.to_owned()).collect();
        Ok(())
    }

    fn require_chunk(&mut self, path: &[u8]) -> Result<Chunk, String> {
        let path = str::from_utf8(path).map_err(|e| e.to_string())?;

        if let Some(chunk) = self.search_file(&path, &self.package_path, ".kailua")? {
            return Ok(chunk);
        }
        if let Some(chunk) = self.search_file(&path, &self.package_path, "")? {
            return Ok(chunk);
        }
        if let Some(chunk) = self.search_file(&path, &self.package_cpath, ".kailua")? {
            return Ok(chunk);
        }
        // avoid loading the native libraries as is

        Err(format!("module not found"))
    }
}

