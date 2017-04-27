extern crate elementtree;

use std;

/// The default target to run if no target was specified and no default target was user-specified.
pub static ALL_TARGETS: &'static str = "_all";

/// Contains data about a project.
pub struct Project {
    pub project_directory: std::path::PathBuf,
    pub include: Vec<std::path::PathBuf>,
    pub targets: std::collections::HashMap<String, String>,
    pub default_target: String,
}

impl Project {
    /// Initializes a new empty project.
    pub fn new() -> Project {
        Project {
            project_directory: std::path::PathBuf::new(),
            include: Vec::new(),
            targets: std::collections::HashMap::new(),
            default_target: "main".to_owned(),
        }
    }

    /// Loads a project from its definition file.
    ///
    /// The definition file is an XML file describing the pro
    pub fn load(&mut self, project_file_path: &std::path::Path) -> Result<(), String> {
        // Load the project's XML file
        let file: std::fs::File = match std::fs::File::open(project_file_path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Unable to open file: {}", e)),
        };
        let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
        let root: elementtree::Element = match elementtree::Element::from_reader(reader) {
            Ok(element) => element,
            Err(e) => return Err(format!("Failed to parse XML: {}", e)),
        };
        self.project_directory = project_file_path.parent().unwrap().to_owned();

        // Load the include paths
        // These specify additional paths to search source files in
        match root.find("include") {
            Some(element) => {
                self.include = element
                    .find_all("path")
                    .map(|path: &elementtree::Element| std::path::PathBuf::from(path.text().trim()))
                    .collect()
            }
            None => (),  // Do not add any additional include paths
        };

        // Load the targets
        // A target consists of a name and a root source file
        let targets: &elementtree::Element = match root.find("targets") {
            Some(element) => element,
            None => return Err("No target definitions found".to_owned()),
        };
        self.targets = targets
            .find_all("target")
            .map(|target: &elementtree::Element| {
                     // TODO(Lyrositor) Handle missing name or src
                     (target.get_attr("name").unwrap().to_owned(),
                      target.get_attr("src").unwrap().to_owned())
                 })
            .collect();

        // Load the default target, replacing it by the default "default target" if not found
        self.default_target = targets
            .get_attr("default")
            .unwrap_or(ALL_TARGETS)
            .to_owned();

        Ok(())
    }

    /// Finds a file within the project, using its include paths to search for it.
    ///
    /// The order of lookup for relative paths:
    /// - current project directory
    /// - including file's directory
    /// - every project include path, in the order in which they were defined, relative to the
    ///   project directory
    /// If the path is absolute, then only an absolute lookup is performed.
    pub fn find_file(&self,
                     filename: &String,
                     dir: &std::path::Path)
                     -> Option<std::path::PathBuf> {
        let mut possible_paths = vec![std::path::PathBuf::from(filename)];
        if possible_paths.first().unwrap().is_relative() {
            possible_paths.push(dir.join(filename));
            possible_paths.extend(self.include
                                      .iter()
                                      .map(|include_path| include_path.join(filename)));
            possible_paths = possible_paths
                .iter()
                .map(|path: &std::path::PathBuf| self.project_directory.join(path))
                .collect();
        }

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}
