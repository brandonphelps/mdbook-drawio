#![allow(unused)]
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};


use regex::{Regex, Captures};

// todo: add caching, each draw-io diagram can take awhile to render
// so we should cache each draw-io diagram and check for modified date time. 
pub struct DrawIo;

impl Preprocessor for DrawIo {
    fn name(&self) -> &str {
        "drawio"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        
        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(DrawIo::add_diagram(chapter).map(|md| {
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

impl DrawIo {
    fn add_diagram(chapter: &mut Chapter) -> Result<String> {
        let updated_content = replace_links(&chapter.content)?;
        log::debug!("exporting the following draw-io images");
        log::debug!("{:?}", updated_content.0);

        // todo: produce warnings / errors if diagrams won't generate
        // nice links,
        // suchas if a page has #@$! in its name, on linus
        // the name would end up with 'diagram-#@$!' and thus not be diretly
        // ref able. 
        
        log::debug!("updated content: {:?}", updated_content.1);
        Ok(updated_content.1)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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

}


