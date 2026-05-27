
//later will check if route has directory_listing true
//in configuration file to call this function
pub fn list_directory(path: &str) -> Vec<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|_| std::fs::read_dir(".").unwrap())
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.path().is_file() {
                    e.file_name().into_string().ok()
                } else {
                    None
                }
            })
        })
        .collect()
}