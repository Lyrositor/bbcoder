extern crate clap;

mod parser;
mod project;

/// The default path to a project's definition file.
static DEFAULT_PROJECT_PATH: &'static str = "project.xml";

/// Processes command-line arguments and builds the desired target.
///
/// The path to the project file can be specified using the `-p` argument.
/// The build target can be optionally specified as the only position argument; if left unspecified,
/// the project's default target will be built.
fn main() {
    // Initialize the argument parser
    let matches = clap::App::new("bbcoder")
        .version("0.1.1")
        .author("Lyrositor")
        .about("BBCode generation tool")
        .arg(clap::Arg::with_name("path")
                .short("p")
                 .long("path")
                 .default_value(DEFAULT_PROJECT_PATH)
                 .help("Path to the BBCoder project file"))
        /*.arg(clap::Arg::with_name("watch")
                 .short("w")
                 .long("watch")
                 .help("Watch for changes to a project and serve the result on port 8080"))*/
        .arg(clap::Arg::with_name("TARGET")
                 .index(1)
                 .help("The target to build"))
        .get_matches();

    // Load the project
    let mut project = project::Project::new();
    let path = std::path::Path::new(matches.value_of("path").unwrap_or_default());
    match project.load(path) {
        Err(e) => {
            println!("ERROR: Invalid project: {}", e);
            std::process::exit(1);
        }
        _ => (),
    };
    if !project.targets.contains_key(&project.default_target) {
        println!("WARNING: Default target '{}' not found",
                 project.default_target);
    }

    // Parse the project and output the BBCode to a file
    let mut target = project.default_target.clone();
    if matches.is_present("TARGET") {
        target = matches.value_of("TARGET").unwrap().to_owned();
    }
    if !project.targets.contains_key(&target) {
        println!("ERROR: Target '{}' not found", &target);
        std::process::exit(1);
    }
    let filename = &project.targets[&target];
    let output_path = std::path::Path::new("target").join(target + ".txt");
    let mut parser = parser::Parser::new(&project);
    match project.find_file(filename, std::path::Path::new(filename).parent().unwrap()) {
        Some(root_path) => {
            match parser.output_bbcode(&root_path, &output_path) {
                Err(e) => println!("ERROR: {}", e),
                _ => (),
            };
        }
        None => {
            println!("ERROR: File '{}' not found", &filename);
            std::process::exit(1);
        }
    };
    std::process::exit(0);
}
