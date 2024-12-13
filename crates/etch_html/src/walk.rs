use std::io;
use std::path::Path;
use walkdir::WalkDir;

pub fn process_html_files<P, F>(directory: P, mut operation: F) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    F: FnMut(&Path, &Path) -> Result<(), io::Error>,
{
    let base_path = directory.as_ref().to_path_buf();
    for entry in WalkDir::new(&base_path).into_iter() {
        let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "html") {
            let relative_path = path
                .strip_prefix(&base_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            operation(path, relative_path)?;
        }
    }
    Ok(())
}


pub async fn process_html_files_async<P, F, Fut>(directory: P, operation: F) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    F: Fn(&Path, &Path) -> Fut,
    Fut: std::future::Future<Output = Result<(), io::Error>>,
{
    let base_path = directory.as_ref().to_path_buf();
    for entry in WalkDir::new(&base_path).into_iter() {
        let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "html") {
            let relative_path = path
                .strip_prefix(&base_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            operation(path, relative_path).await?;
        }
    }
    Ok(())
}