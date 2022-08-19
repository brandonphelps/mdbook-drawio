use std::io;
use clap::{Command, Arg, ArgMatches, crate_version};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};

use mdbook_drawio::DrawIo;

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


// fn main(){
//     env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

//     let matches = Command::new("hello")
//         .arg(Arg::new("drawio-digram"))
//         .arg(Arg::new("output"))
//         .get_matches();

//     let diagram_path: PathBuf = matches.value_of("drawio-digram").unwrap().into();
//     let output_dir: PathBuf = matches.value_of("output").unwrap().into();

//     let args = [diagram_path.to_str().unwrap(),
//                 "--output", ".",
//                 "--output-mode", "absolute"];
//     log::debug!("drawio-exporter.exe {}", args.join(" "));
    
//     let output = process::Command::new("drawio-exporter.exe")
//         .args(&args)
//         .output();

//     match output {
//         Ok(r) => { println!("Success: {:?}", r) },
//         Err(f) => { println!("Failrue: {:?}", f) }
//     };
// }
