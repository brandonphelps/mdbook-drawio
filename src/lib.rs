#![allow(unused)]
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::process;

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
    let start_p_split = start_p_str.split("/").peekable();
    let path_p_split = path_p_str.split("/").peekable();

    let mut uncommon_parts: Vec<String> = vec![];

    println!("Start p");
    // is zip useless? how to do this functionally? 
    loop { 
        let start_n = start_p_split.next();
        let path_n = path_p_split.next();

        // hit the end of the start path, thus we just need to accumlate the end paths
        if start_n.is_none() {
            
        }

        if path_n.is_none() {
            
        }

        println!("{:?}", s);
    }

    start_p_split.peek();


    /*
    if start_p_.peek().is_none() {
        println!("Start is empty");
    }

    if path_p_split.peek().is_none() {
        println!("path is empty");
}
    */

    
    
    todo!()
}



impl DrawIo {
    fn add_diagram(root_dir: &PathBuf, chapter: &mut Chapter) -> Result<String> {
        log::info!("Processing dir: {}", root_dir.to_str().unwrap());
        log::info!("Processing chapter: {}", chapter.name);

        // book root dir is always "."
        // all content is relative to the SUMMARY.md file and hence
        // thus each file a chapter refers to is then located at either "." or w/e the parent dir is.
        let current_dir = std::env::current_dir().unwrap();
        let chapter_path = chapter.source_path.as_ref().unwrap().clone();
        let chapter_parent = chapter_path.parent().unwrap();
        let chapter_abs_path = absolute_path(&chapter_path).unwrap();
        let chapter_dir = chapter_abs_path.parent().unwrap();
        // let relative_path = RelativePathBuf::from_path(&current_dir).unwrap();
        //let new_path = relative_path.to_path(&chapter_abs_path);
        log::debug!("Current dir: {:?}", current_dir.to_str().unwrap());
        log::debug!("Chapter absolute path: {:?}", chapter_abs_path.to_str().unwrap());
        log::debug!("Chapter directory: {:?}", chapter_dir.to_str().unwrap());
        //log::debug!("Rela: {:?}", relative_path.to_string());
        //log::debug!("new: {:?}", new_path.to_str());
        
        
        let updated_content = replace_links(&chapter.content)?;
        log::debug!("exporting the following draw-io images");
        log::debug!("{:?}", updated_content.0);

        // todo: produce warnings / errors if diagrams won't generate
        // nice links,
        // suchas if a page has #@$! in its name, on linus
        // the name would end up with 'diagram-#@$!' and thus not be diretly
        // ref able. 

        // docker run -v$(pwd):/data rlespinasse/drawio-export src/diagram.drawio --output . --format svg
        for diagrams in updated_content.0.iter() {
            // for some reason the output is relative to the file being built???
            log::debug!("diagrams: {:?}", diagrams);
            let diagram_path = chapter_parent.join(diagrams).clean();
            log::debug!("@@digram path: {:?}", diagram_path.to_str().unwrap());
            // log::debug!("docker digram path: {:?}", docker_diagram_path.to_str().unwrap());
            let args = ["run", "-v$(pwd):/data", "rlespinasse/drawio-export", "--output", ".",
                        "--format", "svg", &format!("src/{}", diagrams)];
            log::debug!("Command: {} {:?}", "docker", args.join(" "));
            let output = process::Command::new("docker")
                .args(&args)
                .output();
            match output {
                Ok(r) => {
                    log::debug!("Successful conversion");
                },
                Err(f) => {
                    log::error!("Failed to convert document: {:?}", diagrams);
                }
            }
        }
        log::debug!("updated content: {:?}", updated_content.1);
        Ok(updated_content.1)
    }
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
        println!("Rel path: {:?}", rel_path.to_string());
        println!("New path: {:?}", new_path.to_str().unwrap());
        assert!(false);
    }

    #[test]
    fn relative_path_testing() {
        let path = "hello/world";
        let cur_p = "hello";
        assert_eq!(relative_path(path, cur_p).unwrap(), PathBuf::from("world"));
    }
}


