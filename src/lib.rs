#![allow(unused)]
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::collections::HashMap;
use std::process;
use std::time::Instant;

use path_clean::PathClean;
use regex::{Captures, Regex};
use relative_path::RelativePathBuf;
use std::path::{Path, PathBuf};

mod drawio_cache;

// todo: add caching, each draw-io diagram can take awhile to render
// so we should cache each draw-io diagram and check for modified date time.
pub struct DrawIo {
    // draw io cache. 
    cache: drawio_cache::DrawIoCache,
}

impl DrawIo  {
    pub fn new<P: AsRef<Path>>(path: P) -> DrawIo {
        Self {
            cache: drawio_cache::DrawIoCache::new(path),
        }
    }
}

impl Preprocessor for DrawIo {
    fn name(&self) -> &str {
        "drawio"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // anyway to determine
        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(ref f)) = res {
                log::error!("Error on book! {:?}", f);
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(self.add_diagram(&ctx.root, chapter).map(|md| {
                    chapter.content = md;
                }));
            }
        });
        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

impl DrawIo {
    fn add_diagram(&self, root_dir: &PathBuf, chapter: &mut Chapter) -> Result<String> {
        // root points to the path of book.toml directory.
        log::info!("\n\nProcessing dir: {}", root_dir.to_str().unwrap());
        log::info!("Processing chapter: {}", chapter.name);

        log::debug!(
            "Current dir: {:?}",
            std::env::current_dir().unwrap().to_str().unwrap()
        );

        // book root dir is always "."  all content is relative to thel
        // SUMMARY.md file and hence thus each file a chapter refers
        // to is then located at either "." or w/e the parent dir is.

        let chapter_path = chapter.source_path.as_ref().unwrap().to_path_buf();
        let chapter_dir = PathBuf::from("src")
            .join(chapter_path.parent().unwrap())
            .clean();

        let mut new_content = String::new();
        let regex_v = Regex::new(r"(!\[.*\])\((.*?)-(.*)(\.drawio)\)").unwrap();
        // start index of the chapter.content.
        // this keeps track of what content to keep,
        // so we can replace the link. 
        let mut start_index = 0;

        for entry in regex_v.captures_iter(&chapter.content) {
            let m = entry.get(0).unwrap();
            // named link in the md file [....]
            let md_link = entry.get(1).unwrap().as_str();
            let diagram_name = entry.get(2).unwrap().as_str();
            let page_name = entry.get(3).unwrap().as_str();
            let ext_name = entry.get(4).unwrap().as_str();

            // todo: could have this get deteremined by option
            let new_ext_name = ".svg";
            let expected_key =
                format!("{}-{}{}", diagram_name, page_name, new_ext_name)[2..].to_string();
            let diagram_path = chapter_dir.join(format!("{}.drawio", diagram_name));

            if !diagram_path.is_file() {
                log::error!("Failed to find diagram: {}", diagram_path.to_str().unwrap());
                // since the digrams path specified isn't available
                // skip the entry. 
                continue;
            }

            let f = self.cache.get_diagram(&diagram_path, &expected_key);
            let new_diagrams = match f {
                Ok(r) => { r },
                Err(f) => {
                    let new_diagrams = get_content_from_diagram(&diagram_path).unwrap();
                    for (key, value) in new_diagrams.into_iter() {
                        log::debug!("diagrams: {}", key);
                        self.cache.add_diagram(&diagram_path, &key, &value);
                    }
                    self.cache.get_diagram(&diagram_path,
                                           &expected_key).unwrap()
                }
            };

            new_content += &chapter.content[start_index..m.start()];
            new_content += &new_diagrams;
            start_index = m.end();
        }
        new_content += &chapter.content[start_index..];
        log::debug!("new content: \n{}", new_content);

        Ok(new_content)
    }
}

/// Contains the names of draw io diagrams that have been converted
/// to avoid reconvering if a link is used multiple times.

// pulls out the svg image from a draw io exported xml file.
fn extract_svg<P: AsRef<Path>>(drawio_svg_path: P) -> Option<String> {
    let string = std::fs::read_to_string(drawio_svg_path).unwrap();
    let p = string.find("<svg ").unwrap();
    Some(string[p..].to_owned())
}

fn get_content_from_diagram<P: AsRef<Path>>(
    diagram_path: P,
) -> Result<HashMap<String, String>, &'static str> {
    // assert diagram exists.

    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp_dir.path());
    let p = diagram_path.as_ref().to_str().unwrap();
    log::debug!("Converting: {}", p);
    let args = [
        p,
        // todo: adjust output directory based on path for where the chapter expects it.
        "--output",
        temp_dir.path().to_str().unwrap(),
        "--format",
        "svg",
        "--output-mode",
        "absolute",
    ];
    log::debug!("drawio-exporter.exe {}", args.join(" "));

    let mut results = HashMap::new();
    let output = process::Command::new("drawio-exporter.exe")
        .args(&args)
        .output();
    match output {
        Ok(r) => {
            for dir in std::fs::read_dir(temp_dir.path()).unwrap() {
                if let Ok(e) = dir {
                    if e.path().is_file() {
                        let filename: String =
                            e.path().file_name().unwrap().to_str().unwrap().to_string();

                        log::debug!("Converted {}", filename);
                        results.insert(filename, extract_svg(e.path()).unwrap());
                    } else {
                        log::debug!("Is not a file: {}", e.path().to_str().unwrap());
                    }
                } else {
                    log::debug!("Dir not okay");
                }
            }
            log::debug!(
                "Successful conversion: {}",
                String::from_utf8(r.stdout).unwrap()
            );
        }
        Err(f) => {
            log::error!("Failed conversion: {:?}", f);
            // todo: how to return an error?????
            log::error!("purpose to make error");
            panic!()
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::path::PathBuf;

    #[test]
    fn replace_link_test() {
        let expected_content = r#"
hello world
![blahalala](testdiagram-Page-1.drawio)
blkafjaklfj
"#;

        let mut new_content = String::new();
        let regex_v = Regex::new(r"(!\[.*\])\((.*)-(.*)(\.drawio)\)").unwrap();
        let mut start_index = 0;

        let resources_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");

        // keys are pages.svg, values are the svg.
        let mut diagrams = HashMap::new();
        for entry in regex_v.captures_iter(expected_content) {
            let m = entry.get(0).unwrap();
            let md_link = entry.get(1).unwrap().as_str();
            let diagram_name = entry.get(2).unwrap().as_str();
            let page_name = entry.get(3).unwrap().as_str();
            // todo: could have this get deteremined by option
            let new_ext_name = ".svg";
            let expected_key = format!("{}-{}{}", diagram_name, page_name, new_ext_name);
            let new_diagrams =
                get_content_from_diagram(resources_dir.join("testdiagram.drawio")).unwrap();
            for (key, value) in new_diagrams.into_iter() {
                // println!("keys: {}", key);
                diagrams.insert(key, value);
            }

            println!("Custom Key {}", &expected_key[2..]);
            println!("Value: {}", diagrams.get(&expected_key).unwrap());

            new_content += &expected_content[start_index..m.start()];
            new_content += &diagrams.get(&expected_key).unwrap();
            start_index = m.end();
        }
        new_content += &expected_content[start_index..];
        println!("new content: \n{}", new_content);
        assert!(false)
    }


}
