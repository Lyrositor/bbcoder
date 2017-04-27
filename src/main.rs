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
                 .help("The target to build (default: all)"))
        .get_matches();

    // Load the project
    let mut proj = project::Project::new();
    let path = std::path::Path::new(matches.value_of("path").unwrap_or_default());
    match proj.load(path) {
        Err(e) => {
            println!("ERROR: Invalid project: {}", e);
            std::process::exit(1);
        }
        _ => (),
    };
    if !proj.targets.contains_key(&proj.default_target) &&
       proj.default_target != project::ALL_TARGETS {
        println!("WARNING: Default target '{}' not found",
                 proj.default_target);
    }

    // Build all the desired targets
    let mut target_name = proj.default_target.clone();
    if matches.is_present("TARGET") {
        target_name = matches.value_of("TARGET").unwrap().to_owned();
    }
    if target_name == project::ALL_TARGETS {
        for target in proj.targets.keys() {
            build_target(&proj, target);
        }
    } else if !proj.targets.contains_key(&target_name) {
        println!("ERROR: Target '{}' not found", &target_name);
        std::process::exit(1);
    } else {
        build_target(&proj, &target_name);
    }
    std::process::exit(0);
}

/// Parses a project's target and outputs the BBCoded to a file
fn build_target(project: &project::Project, target: &String) {
    let filename = &project.targets[target];
    let output_path = std::path::Path::new("target").join(format!("{}.txt", target));
    let mut parser = parser::Parser::new(&project);
    match project.find_file(filename, std::path::Path::new(filename).parent().unwrap()) {
        Some(root_path) => {
            match parser.output_bbcode(&root_path, &output_path) {
                Err(e) => {
                    println!("ERROR: {}", e);
                    std::process::exit(1);
                }
                _ => (),
            };
        }
        None => {
            println!("ERROR: File '{}' not found", &filename);
            std::process::exit(1);
        }
    };
}
