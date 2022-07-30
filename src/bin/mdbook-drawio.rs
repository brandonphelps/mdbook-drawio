#![feature(absolute_path)]

use std::io;
use clap::{Command, Arg, ArgMatches, crate_version};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};

use mdbook_drawio::DrawIo;


/*
fn old_main() {

    let matches = clap::Command::new("mdbook-drawio")
        // todo: use cargo feature. 
        .version("0.1.0")
        .arg(clap::Arg::new("book-path").required(true))
        .arg(clap::Arg::new("output-dir"))
        .get_matches();

    let book_root: PathBuf = matches
        .value_of("book-path")
        .expect("did specify boot root").into();

    let out_dir: PathBuf = matches.value_of("output-dir")
        .map_or(tempdir().unwrap().into_path(), |f| f.into());

    println!("copying to: {:?}", out_dir.to_str().unwrap());

    // book_root = /home/brandon/projects/rust/address_space/docs
    // out_Dir = /tmp/.tmp73V2lm/
    let actual_out = out_dir.join(PathBuf::from(book_root.file_name().unwrap()));
    // actual_out = out_dir / docs. 
    println!("Actual out: {:?}", actual_out.to_str().unwrap());

    let mut options = CopyOptions::new();
    options.copy_inside = false;
    fs_extra::dir::copy(&book_root.join("."), &out_dir, &options).unwrap();
    let mut book = MDBook::load(&actual_out).unwrap();

    // copy the mdbook to a temporary output directory,
    // load up the mdbook pointing at the new output directory.
    // scan across all the mdbook contents checking for
    //     links to drawio diagrams.
    // do analysis to determine if generating the book is possible.
    //     check for 
*/
    /*
    book.book.for_each_mut(|chapter: &mut BookItem| {
        match chapter { 
            BookItem::Chapter(ref mut c) => {
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
                c.content = "hello world".to_string();
            } 
            _ => { }
        }
    });

    // todo: check if user provides an output directory via toml file. 

    let output_build_dir = std::path::absolute(book_root.join("book")).unwrap();

    println!("Building dir: {:?}", book.config.build.build_dir);
    println!("building at: {:?}", output_build_dir.to_str().unwrap());
    book.config.build.build_dir = output_build_dir;
    book.build().unwrap();
    println!("Hello, world!");

}
    */
fn make_app() -> clap::Command<'static> {
    Command::new("mdbook-drawio")
        .version(crate_version!())
        .about("mdbook preprocessor to add draw io support")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
        .subcommand(
            Command::new("install")
                .arg(Arg::new("dir")
                     .default_value(".")
                     .help("Root directory for the book,\nshould contain the configuration file (`book.toml`)"))
                .about("Install the required asset files and include it in the config"),
        )
}

fn handle_supports(sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = DrawIo.supports_renderer(renderer);

    if supported {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}


fn handle_preprocessing() -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: the mdbook-drawio preprocessor was built against version \
             {} of mdbook, but we're being called from version {}",
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = DrawIo.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let matches = make_app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(sub_args);
    } else if let Err(e) = handle_preprocessing() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}















