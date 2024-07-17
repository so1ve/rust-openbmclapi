use std::path::Path;

pub fn path_basename(path: &str) -> Option<&str> {
    Path::new(path).file_name().unwrap().to_str()
}
