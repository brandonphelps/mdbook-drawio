#![allow(unused)]
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::process;
use std::collections::HashMap;

use path_clean::PathClean;
use relative_path::RelativePathBuf;
use std::path::{Path, PathBuf};
use regex::{Regex, Captures};

// todo: add caching, each draw-io diagram can take awhile to render
// so we should cache each draw-io diagram and check for modified date time. 
pub struct DrawIo;

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
                res = Some(DrawIo::add_diagram(&ctx.root, chapter).map(|md| {
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

/// transforms a chapters content extracting drawio links
/// to be exported. 
// vector of links that are to be used for exporting.
// new content page. 
fn replace_links(content: &str) -> Result<(Vec<String>, String)> {
    let regex_v = Regex::new(r"(!\[.*\])\((.*)-(.*)(\.drawio)\)").unwrap();
    let mut links = vec![];

    let result = regex_v.replace_all(content, |caps: &Captures| {
        let md_link = caps.get(1).unwrap().as_str();
        let diagram_name = caps.get(2).unwrap().as_str();
        let page_name = caps.get(3).unwrap().as_str();
        let ext_name = caps.get(4).unwrap().as_str();
        // todo: could have this get deteremined by option 
        let new_ext_name = ".svg";
        log::debug!("leading link: {}", md_link);
        log::debug!("{} {}", "name of diagram: ", diagram_name);
        log::debug!("{} {}", "name of page: ", page_name);
        log::debug!("{} {}", "name of extension: ", ext_name);

        log::debug!("new link {}{}", md_link,
                 format!("({}-{}{})", diagram_name, page_name, new_ext_name));
        links.push(format!("{}{}", diagram_name, ext_name));
        format!("{}({}-{}{})", md_link, diagram_name, page_name, new_ext_name)
    });
    Ok((links, result.to_string()))
}

fn absolute_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }.clean();
    Ok(absolute_path)
}

/// Computes a relative, if possible, from path to start.
/// this attempts to be similar in functionality to that of pythons os.path.relpath.
fn relative_path(path: impl AsRef<Path>, start: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let start_p = absolute_path(start)?;
    let path_p = absolute_path(path)?;
    let start_p_str = start_p.to_str().unwrap();
    let path_p_str = path_p.to_str().unwrap();
    // todo: determine if there is an os.seperator()
    let mut start_p_split = start_p_str.split("/").peekable();
    let mut path_p_split = path_p_str.split("/").peekable();

    let mut uncommon_parts: Vec<String> = vec![];

    // 
    // path: /hello/world/i/like
    // start: /hello/world
    // result: i/like
    
    // 
    // path: /hello/world/
    // start: /hello/world/i/like
    // result: ../..

    // 
    // path: /hello/world/hello/world
    // start: /hello/world/i/like
    // result: ../../hello/world

    // seems completely uncessary to perform all this path manip stuff.
    let mut start_n = start_p_split.next();
    let mut path_n = path_p_split.next();

    // is zip useless? how to do this functionally? 
    loop { 
        start_n = start_p_split.next();
        path_n = path_p_split.next();

        if start_n != path_n || start_n.is_none() || path_n.is_none() {
            break;
        } 
    }

    if let Some(n) = start_n {
        uncommon_parts.push("..".to_string());
    }        

    while let Some(start_n) = start_p_split.next() {
        uncommon_parts.push("..".to_string());
    }

    if let Some(n) = path_n {
        uncommon_parts.push(n.to_string());
    }        

    while let Some(path_n) = path_p_split.next() {
        uncommon_parts.push(path_n.to_string());
    }

    if uncommon_parts.is_empty() {
        uncommon_parts.push(String::from("."));
    }
    let result = uncommon_parts.join("/");
    let result_path = PathBuf::from(result).clean();
    Ok(result_path)
}



impl DrawIo {
    fn add_diagram(root_dir: &PathBuf, chapter: &mut Chapter) -> Result<String> {
        // root points to the path of book.toml directory.
        log::info!("\n\nProcessing dir: {}", root_dir.to_str().unwrap());
        log::info!("Processing chapter: {}", chapter.name);

        log::debug!("Current dir: {:?}",
                    std::env::current_dir().unwrap().to_str().unwrap());

        // book root dir is always "."  all content is relative to thel
        // SUMMARY.md file and hence thus each file a chapter refers
        // to is then located at either "." or w/e the parent dir is.

        let chapter_path = chapter.source_path.as_ref().unwrap().to_path_buf();
        // log::debug!("Chapter path: {:?}", chapter_path.to_str().unwrap());
        // assert!(chapter_path.is_file());
        // let chapter_dir = chapter_path.parent().unwrap();


        let mut new_content = String::new();
        



        log::debug!("chapter directory: {:?}", chapter_path.to_str().unwrap());
        let updated_content = replace_links(&chapter.content)?;

        // todo: produce warnings / errors if diagrams won't generate
        // nice links,
        // suchas if a page has #@$! in its name, on linus
        // the name would end up with 'diagram-#@$!' and thus not be diretly
        // ref able. 

        // docker run -v$(pwd):/data rlespinasse/drawio-export
        // src/diagram.drawio --output . --format svg
        for diagrams in updated_content.0.iter() {
            // for some reason the output is relative to the file being built???
            log::debug!("diagrams: {:?}", diagrams);
            let diagram_path: PathBuf = absolute_path(&chapter_path).unwrap()
                .parent().unwrap().to_path_buf();
            log::debug!("Diagram path: {:?}", diagram_path.clean().to_str().unwrap());

            //assert!(diagram_path.is_file());
            let args = [diagram_path.to_str().unwrap(),
                        // todo: adjust output directory based on path for where the chapter expects it. 
                        "--output", "temp_file.svg",
                        "--format", "svg",
                        "--output-mode", "absolute"];
            log::debug!("drawio-exporter.exe {}", args.join(" "));
            
            let output = process::Command::new("drawio-exporter.exe")
                .args(&args)
                .output();
            match output {
                Ok(r) => {

                    let svg_output = std::fs::read_to_string("temp_file.svg");
                    
                    log::debug!("Successful conversion: {}", String::from_utf8(r.stdout).unwrap());
                },
                Err(f) => {
                    log::error!("Failed conversion: {:?}", f);
                    // todo: how to return an error?????
                    println!("purpose to make error");
                    panic!()
                }
            }
        }
        log::debug!("updated content: {:?}", updated_content.1);
        Ok(updated_content.1)
    }
}

// pulls out the svg image from a draw io exported xml file. 
fn extract_svg<P: AsRef<Path>>(drawio_svg_path: P) -> Option<String> {
    let string = std::fs::read_to_string(drawio_svg_path).unwrap();
    let p = string.find("<svg ").unwrap();
    Some(string[p..].to_owned())
}

fn get_content_from_diagram<P: AsRef<Path>>(diagram_path: P) -> Result<HashMap<String, String>, &'static str> {
    // assert diagram exists.

    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp_dir.path());
    let args = [diagram_path.as_ref().to_str().unwrap(),
                // todo: adjust output directory based on path for where the chapter expects it. 
                "--output", temp_dir.path().to_str().unwrap(),
                "--format", "svg",
                "--output-mode", "absolute"];
    log::debug!("drawio-exporter.exe {}", args.join(" "));

    let mut results = HashMap::new();
    let output = process::Command::new("drawio-exporter.exe")
        .args(&args)
        .output();
    match output {
        Ok(r) => {
            for dir in std::fs::read_dir(temp_dir.path()).unwrap() {
                if let Ok(e) = dir {
                    if e.path().is_file(){
                        let filename: String = e.path().file_name().unwrap().to_str().unwrap().to_string();
                        results.insert(filename,
                                       extract_svg(e.path()).unwrap());
                    }
                }
            }
            log::debug!("Successful conversion: {}", String::from_utf8(r.stdout).unwrap());
        },
        Err(f) => {
            println!("Failed conversion: {:?}", f);
            // todo: how to return an error?????
            println!("purpose to make error");
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
    fn test_link_extraction() {
        let content = r#"
hello world
![blahalala](blah-page1.drawio)
blkafjaklfj
"#;

        let expected_content = r#"
hello world
![blahalala](blah-page1.svg)
blkafjaklfj
"#;

        let new_content = replace_links(content).unwrap();

        assert_eq!(new_content.0, vec!["blah.drawio"]);
        assert_eq!(new_content.1, expected_content);
    }

    #[test]
    fn test_link_extraction_mutli() {
        let content = r#"
hello world
![blahalala](blah-page1.drawio)
![blahalala](diagram-Woooo.drawio)
blkafjaklfj
"#;

        let expected_content = r#"
hello world
![blahalala](blah-page1.svg)
![blahalala](diagram-Woooo.svg)
blkafjaklfj
"#;

        let new_content = replace_links(content).unwrap();

        assert_eq!(new_content.0, vec!["blah.drawio", "diagram.drawio"]);
        assert_eq!(new_content.1, expected_content);
    }

    #[test]
    fn parent_paths() {
        let path = PathBuf::from("./chapter.md");
        let parent = path.parent().unwrap();

        assert_eq!(parent, PathBuf::from("."));
            
        let rel_path = RelativePathBuf::from_path("brandon").unwrap();
        let new_path = rel_path.to_path("/home/brandon");
        assert!(false);
    }

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

        let mut diagrams = HashMap::new();

        // let chapter_dir = "hello";
        for entry in regex_v.captures_iter(expected_content) {
            let m = entry.get(0).unwrap();
            let md_link = entry.get(1).unwrap().as_str();
            let diagram_name = entry.get(2).unwrap().as_str();
            let page_name = entry.get(3).unwrap().as_str();
            let ext_name = entry.get(4).unwrap().as_str();
            // todo: could have this get deteremined by option 
            let new_ext_name = ".svg";
            // println!("leading link: {}", md_link);
            // println!("{} {}", "name of diagram: ", diagram_name);
            // println!("{} {}", "name of page: ", page_name);
            // println!("{} {}", "name of extension: ", ext_name);
            let expected_key = format!("{}-{}{}", diagram_name, page_name, new_ext_name);
            let new_diagrams = get_content_from_diagram(resources_dir.join("testdiagram.drawio")).unwrap();
            for (key, value) in new_diagrams.into_iter() {
                // println!("keys: {}", key);
                diagrams.insert(key, value);
            }

            // println!("new link {}{}", md_link,
            //             format!("({}-{}{})", diagram_name, page_name, new_ext_name));

            println!("Key {}", expected_key);
            println!("Value: {}", diagrams.get(&expected_key).unwrap());

            new_content += &expected_content[start_index..m.start()];
            new_content += &diagrams.get(&expected_key).unwrap();
            start_index = m.end();
        }
        new_content += &expected_content[start_index..];
        println!("new content: \n{}", new_content);
        assert!(false)
    }


    #[test]
    fn relative_path_testing() {
        let path = "hello/world";
        let cur_p = "hello/world/blah/balh";

        let expected_result = PathBuf::from("../../");
        assert_eq!(relative_path(path, cur_p).unwrap(), expected_result);

        let path = "hello/world/balh";
        let cur_p = "hello/world/blah/balh";

        let expected_result = PathBuf::from("../../balh");
        assert_eq!(relative_path(path, cur_p).unwrap(), expected_result);

        let path = "hello/world/balh/jojo/bizaar";
        let cur_p = "hello/world/blah/balh";

        let expected_result = PathBuf::from("../../balh/jojo/bizaar");
        assert_eq!(relative_path(path, cur_p).unwrap(), expected_result);

        let path = "hello/jojo/bizaar";
        let cur_p = "hello/world/blah/balh";

        let expected_result = PathBuf::from("../../../jojo/bizaar");
        assert_eq!(relative_path(path, cur_p).unwrap(), expected_result);

        let path = "jojo/hello";
        let cur_p = "hello/world/blah/balh";

        let expected_result = PathBuf::from("../../../../jojo/hello");
        assert_eq!(relative_path(path, cur_p).unwrap(), expected_result);
    }
}


