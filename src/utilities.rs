
fn absolute_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }
    .clean();
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

        log::debug!(
            "new link {}{}",
            md_link,
            format!("({}-{}{})", diagram_name, page_name, new_ext_name)
        );
        links.push(format!("{}{}", diagram_name, ext_name));
        format!(
            "{}({}-{}{})",
            md_link, diagram_name, page_name, new_ext_name
        )
    });
    Ok((links, result.to_string()))
}



#[cfg(test)]
mod tests {
    use super::*;

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
