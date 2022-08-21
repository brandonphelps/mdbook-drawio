use std::time::SystemTime;
use std::path::Path;
use std::path::PathBuf;

// get this file from the cahce
// file points to the drawio digram and page.
// cache does a check to see if there is a file.
//  cache checks to see if the files modified date time
//    is older than that of the corresponding draw io diagram.

// exporting diagrams is slow, so create
// cache to store exported contents.
pub struct DrawIoCache {
    // uses the file system to act a store
    // this is  due to avoiding slowdown
    // during serving the files and a user changes
    // a single md and does not touch the draw io files.
    root_dir: PathBuf,
}

impl DrawIoCache {

    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        log::debug!("Placing cache at: {}", root_dir.as_ref().to_str().unwrap());
        Self {
            root_dir: root_dir.as_ref().into(),
        }
    }

    fn get_diagram_cache_path<P: AsRef<Path>>(&self, path: P, page: &str) -> PathBuf {
        if path.as_ref().is_absolute() {
            panic!("blah");
        }
        self.root_dir.join(path.as_ref()).join(page)
    }

    // None if the item does not exist. 
    pub fn get_time<P: AsRef<Path>>(&self, path: P, page: &str) -> Option<SystemTime> {
        
        let d_path = self.get_diagram_cache_path(path, page);
        log::debug!("Getting metadata for: {}", d_path.to_str().unwrap());
        match std::fs::metadata(d_path) {
            Ok(r) => { Some(r.modified().unwrap()) },
            _ => { None }
        }
    }

    // all paths should be relative to the context of the running tool.
    // path both specifies the draw io diagram to get, with the page being
    // the sub entry. 
    pub fn get_diagram<P: AsRef<Path>>(&self, path: P, page: &str) -> Result<String, String> {
        log::debug!("Getting diagram from {} - {}", path.as_ref().to_str().unwrap(), page);

        let metadata = std::fs::metadata(&path).unwrap();

        match metadata.modified() {
            Ok(r) => {
                if let Some(diagram_time) = self.get_time(&path, &page) {
                    if r > diagram_time {
                        self.clear_diagram(&path, &page)
                    }
                }
            },
            _ => {
            }
        };

        let d_path = self.get_diagram_cache_path(path, page);
        if d_path.is_file() {
            // load the file and return the exported contents. 
            Ok(std::fs::read_to_string(d_path).unwrap())
        } else {
            Err(format!("no entry in cache for {}", page))
        }
    }

    /// removes entry from cache. 
    pub fn clear_diagram<P: AsRef<Path>>(&self, path: P, page: &str) {
        let d_path = self.get_diagram_cache_path(path, page);
        if d_path.is_file() {
            std::fs::remove_file(d_path);
        }
    }

    /// add an entry into the cache. 
    pub fn add_diagram<P: AsRef<Path>>(&self, path: P, page: &str, content: &str) -> Result<(), String> {
        let d_path = self.get_diagram_cache_path(path, page);
        log::debug!("Adding diagram {} - {}", d_path.to_str().unwrap(), page);
        std::fs::create_dir_all(d_path.parent().unwrap());
        std::fs::write(d_path, content).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn cache_test() {
        let d_root_dir = PathBuf::from("resources");
        let resources_dir = d_root_dir; // PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");



        let temp_dir = tempfile::tempdir().unwrap();
        let mut draw_io_cache = DrawIoCache::new(&temp_dir.path());

        // cache miss.
        let f = draw_io_cache.get_diagram(resources_dir.join("testdiagram.drawio"), "page1");
        assert!(f.is_err());

        draw_io_cache.add_diagram(resources_dir.join("testdiagram.drawio"), "page1", "hello world");

        let f = draw_io_cache.get_diagram(resources_dir.join("testdiagram.drawio"), "page1");
        assert!(f.is_ok());
        assert_eq!(f, Ok("hello world".to_string()));
    }
}
