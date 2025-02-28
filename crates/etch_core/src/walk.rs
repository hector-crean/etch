use std::collections::HashSet;
use std::io;
use std::path::Path;
use etch_tsx::file::TsxError;
use walkdir::WalkDir;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileWalkerError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),
    #[error(transparent)]
    TsxError(#[from] TsxError),
}

pub struct FileWalker {
    allowed_extensions: HashSet<String>,
}

impl FileWalker {
    pub fn new(extensions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            allowed_extensions: extensions.into_iter().map(Into::into).collect(),
        }
    }

    pub fn visit<P, F>(&self, directory: P, mut operation: F) -> Result<(), FileWalkerError>
    where
        P: AsRef<Path>,
        F: FnMut(&Path, &Path) -> Result<(), FileWalkerError>,
    {
        let base_path = directory.as_ref().to_path_buf();
        for entry in WalkDir::new(&base_path).into_iter() {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && self.has_allowed_extension(path) {
                let relative_path = path.strip_prefix(&base_path)?;
                operation(path, relative_path)?;
            }
        }
        Ok(())
    }

    pub async fn visit_async<P, F, Fut>(&self, directory: P, operation: F) -> Result<(), FileWalkerError>
    where
        P: AsRef<Path>,
        F: Fn(&Path, &Path) -> Fut,
        Fut: std::future::Future<Output = Result<(), FileWalkerError>>,
    {
        let base_path = directory.as_ref().to_path_buf();
        for entry in WalkDir::new(&base_path).into_iter() {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && self.has_allowed_extension(path) {
                let relative_path = path.strip_prefix(&base_path)?;
                operation(path, relative_path).await?;
            }
        }
        Ok(())
    }

    fn has_allowed_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map_or(false, |ext| self.allowed_extensions.contains(ext))
    }
}

