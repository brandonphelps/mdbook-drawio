#![feature(absolute_path)]

use std::path::PathBuf;
use mdbook::{MDBook, BookItem};
use tempfile::tempdir;

use fs_extra::dir::CopyOptions;

struct DrawIOConvert {
    /// Path to where the drawio diagram is referenced.
    // relative to the root of the book.
    link: String, 

    /// Page entry, indicate which page of the
    /// drawio diagram is to be replaced. 
    page: String, 
}

fn main() {

    let regex = regex::Regex::new(r"!\[.*\]\((.*.drawio)\)").unwrap();

    let matches = clap::Command::new("mdbook-drawio")
        // todo: use cargo feature. 
        .version("0.1.0")
        .arg(clap::Arg::new("book-path").required(true))
        .arg(clap::Arg::new("output-dir"))
        .get_matches();

    let book_root: PathBuf = matches.value_of("book-path").expect("did specify boot root").into();
    let out_dir: PathBuf = matches.value_of("output-dir")
        .map_or(tempdir().unwrap().into_path(), |f| f.into());

    println!("copying to: {:?}", out_dir.to_str().unwrap());

    // todo: get base path of book_root.
    let actual_out = out_dir.join(PathBuf::from(book_root.file_name().unwrap()));

    println!("Actual out: {:?}", actual_out.to_str().unwrap());

    let mut options = CopyOptions::new();
    options.copy_inside = false;
    fs_extra::dir::copy(&book_root.join("."), &out_dir, &options).unwrap();
    let book = MDBook::load(&actual_out).unwrap();

    // copy the mdbook to a temporary output directory,
    // load up the mdbook pointing at the new output directory.
    // scan across all the mdbook contents checking for
    //     links to drawio diagrams.
    // do analysis to determine if generating the book is possible.
    //     check for 

    for chapter in book.iter() {
        match chapter { 
            BookItem::Chapter(c) => {
                for entry in regex.captures_iter(&c.content){
                    println!("Base: {:?}", entry);
                    println!("Entry: '{:?}'", entry.get(1).unwrap().as_str());
                    let diagram_link: &str = entry.get(1).unwrap().as_str();

                    let path_regex = regex::Regex::new(r"(.*)-(.*).drawio").unwrap();
                    let diagram_path_str = path_regex.captures(diagram_link).unwrap();
                    let temp = String::from(diagram_path_str.get(1).unwrap().as_str());
                    let diagram_path = PathBuf::from(temp + ".drawio");

                    let full_path = std::path::absolute(actual_out.join("src")
                                                        .join(diagram_path)).unwrap();

                    if ! full_path.is_file() {
                        println!("Error path to draw io diagram is not a file: {:?}",
                                 full_path.as_os_str());
                    } else {
                        println!("Found an entry {:?}", full_path.as_os_str());
                    }

                }
            } 
            _ => { }
        }
    }


    println!("Hello, world!");
}
