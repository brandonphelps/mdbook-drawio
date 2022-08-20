use super::DrawIoFile;
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

    // all paths should be relative to the context of the running tool.
    pub fn get_diagram<P: AsRef<Path>>(&self, path: P, page: &str) -> Result<String, String> {
        let d_path = self.get_diagram_cache_path(path, page);
        if d_path.is_file() {
            // load the file and return the exported contents. 
            Ok(std::fs::read_to_string(d_path).unwrap())
        } else {
            Err(format!("no entry in cache for {}", page))
        }
    }

    /// add an entry into the cache. 
    pub fn add_diagram<P: AsRef<Path>>(&self, path: P, page: &str, content: &str) -> Result<(), String> {
        let d_path = self.get_diagram_cache_path(path, page);
        println!("{:?}", d_path.to_str().unwrap());
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
