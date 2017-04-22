extern crate elementtree;
extern crate regex;

use project;
use std;
use std::io::Write;

/// A BBXML parser which can be used to convert it to BBCode.
pub struct Parser<'a> {
    classes: std::collections::HashMap<String, String>,
    project: &'a project::Project,
    templates: std::collections::HashMap<String, elementtree::Element>,
}

impl<'a> Parser<'a> {
    /// Initializes a new empty parser.
    pub fn new(project: &'a project::Project) -> Parser<'a> {
        Parser {
            classes: std::collections::HashMap::new(),
            project: project,
            templates: std::collections::HashMap::new(),
        }
    }

    /// Parses a BBXML file and outputs its body as BBCode.
    pub fn output_bbcode(&mut self,
                         root_path: &std::path::Path,
                         output_path: &std::path::Path)
                         -> Result<(), String> {
        // Process the root file for its includes, templates and paths
        self.process_file(root_path)?;

        // Convert this file's body to BBCode
        // TODO(Lyrositor) Re-parsing this file is a bit of a waste, since we have already done it
        let root: elementtree::Element = elementtree::Element::from_reader(
            std::io::BufReader::new(std::fs::File::open(root_path).unwrap())).unwrap();
        match root.find("body") {
            Some(body) => {
                match std::fs::create_dir_all(output_path.parent().unwrap()) {
                    Err(e) => return Err(format!("Failed to create directory: {}", e)),
                    _ => (),
                }
                let mut output = std::io::BufWriter::new(std::fs::File::create(output_path)
                                                             .unwrap());
                self.parse_element(body, &mut output, &std::collections::HashMap::new())?;

            }
            None => return Err("No body was found in target root".to_owned()),
        }

        Ok(())
    }

    /// Processes a single file for its classes and templates.
    fn process_file(&mut self, file_path: &std::path::Path) -> Result<(), String> {
        let filename = file_path.to_str().unwrap();

        // Open the XML file
        let file: std::fs::File = match std::fs::File::open(file_path) {
            Ok(file) => file,
            Err(e) => return Err(format!("'{}': Unable to open file: {}", filename, e)),
        };
        let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
        let root: elementtree::Element = match elementtree::Element::from_reader(reader) {
            Ok(element) => element,
            Err(e) => return Err(format!("'{}': Failed to parse XML: {}", filename, e)),
        };
        if root.tag().name() != "bbxml" {
            return Err(format!("'{}': Not a bbxml file, invalid root tag", filename));
        }

        // Process this file's includes
        match self.process_includes(&root, &file_path.parent().unwrap()) {
            Err(e) => return Err(format!("'{}': {}", filename, e)),
            _ => (),
        }

        // Process this file's classes
        match root.find("classes") {
            Some(classes) => {
                match self.process_classes(&classes) {
                    Err(e) => return Err(format!("'{}': {}", filename, e)),
                    _ => (),
                }
            }
            None => (),  // No classes defined in this file
        }

        // Process this file's templates
        match root.find("templates") {
            Some(templates) => {
                match self.process_templates(&templates) {
                    Err(e) => return Err(format!("'{}': {}", filename, e)),
                    _ => (),
                }
            }
            None => (),  // No templates defined in this file
        }

        Ok(())
    }

    /// Processes all included files individually, adding their classes and templates.
    ///
    // TODO(Lyrositor) This is probably vulnerable to circular dependencies.
    fn process_includes(&mut self,
                        bbxml: &elementtree::Element,
                        dir: &std::path::Path)
                        -> Result<(), String> {
        for include in bbxml.find_all("include") {
            // Ensure the attribute is there
            match include.get_attr("src") {
                Some(src) => {
                    // Attempt to locate the file
                    match self.project.find_file(&src.to_owned(), dir) {
                        Some(path) => {
                            match self.process_file(&path) {
                                Err(e) => return Err(e),
                                _ => (),
                            }
                        }
                        None => return Err(format!("File '{}' not found", &src)),
                    }
                }
                None => return Err("Missing 'src' attribute in include".to_owned()),
            }
        }

        Ok(())
    }

    /// Processes a list of classes, storing their content in a more compact form.
    ///
    /// No attempt is made to inspect the content of classes, so they could be any arbitrary
    /// character data; only newlines are replaced (with whitespace).
    fn process_classes(&mut self, classes: &elementtree::Element) -> Result<(), String> {
        for class in classes.find_all("class") {
            match class.get_attr("name") {
                Some(name) => {
                    match self.classes
                              .insert(name.to_owned(),
                                      Parser::compact_text(class.text(), " ")) {
                        _ => (),
                    }
                }
                None => return Err("Missing 'name' attribute in class".to_owned()),
            }
        }
        Ok(())
    }

    /// Processes a list of templates, storing their content as an XML element.
    fn process_templates(&mut self, templates: &elementtree::Element) -> Result<(), String> {
        for template in templates.find_all("template") {
            match template.get_attr("name") {
                Some(name) => {
                    match self.templates.insert(name.to_owned(), template.clone()) {
                        _ => (),
                    }
                }
                None => return Err("Missing 'name' attribute in template".to_owned()),
            }
        }
        Ok(())
    }

    /// Main parsing function, parses an XML element to convert it to BBCode.
    fn parse_element(&mut self,
                     element: &elementtree::Element,
                     output: &mut std::io::BufWriter<std::fs::File>,
                     replacements: &std::collections::HashMap<String, elementtree::Element>)
                     -> Result<(), String> {
        self.output_text(element.text(), output, replacements)?;
        for child in element.children() {
            match child.tag().name() {
                "br" => {
                    match output.write("\n".as_bytes()) {
                        Err(e) => return Err(format!("Failed to write to output: {}", e)),
                        _ => (),
                    }
                    self.output_text(child.tail(), output, replacements)?;
                }
                "include" => {
                    match child.get_attr("template") {
                        Some(template_name) => {
                            let mut include_replacements:
                            std::collections::HashMap<String, elementtree::Element> =
                                replacements.clone();
                            for param in child.find_all("param") {
                                match param.get_attr("name") {
                                    Some(name) => {
                                        include_replacements.insert(name.to_owned(), param.clone());
                                        ()
                                    }
                                    None => {
                                        return Err("Missing 'name' attribute in param".to_owned())
                                    }
                                }
                            }
                            let template = match self.templates.get(template_name) {
                                    Some(template) => template,
                                    None => {
                                        return Err(format!("Template '{}' not found",
                                                           template_name))
                                    }
                                }
                                .clone();
                            self.parse_element(&template, output, &include_replacements)?;
                            ()
                        }
                        None => return Err(format!("Missing 'template' attribute in include")),
                    }
                }
                "li" => {
                    match write!(output, "[*]") {
                        Err(e) => return Err(format!("Failed to write to output: {}", e)),
                        _ => (),
                    }
                    self.parse_element(child, output, replacements)?;
                }
                _ => {
                    // Craft the tag's option attribute
                    let mut options: Vec<String> = Vec::new();
                    match child.get_attr("class") {
                        Some(classes) => {
                            for class in classes.split_whitespace() {
                                match self.classes.get(class) {
                                    Some(class_body) => options.push(class_body.clone()),
                                    None => (),
                                };
                            }
                        }
                        None => (),
                    }
                    match child.get_attr("option") {
                        Some(option) => options.push(option.to_owned()),
                        None => (),
                    }

                    // Create the opening tag
                    match write!(output, "[{}", child.tag().name().to_uppercase()) {
                        Err(e) => return Err(format!("Failed to write to output: {}", e)),
                        _ => (),
                    }
                    if !options.is_empty() {
                        match write!(output, "={}", options.join("").trim()) {
                            Err(e) => return Err(format!("Failed to write to output: {}", e)),
                            _ => (),
                        }
                    }
                    match write!(output, "]") {
                        Err(e) => return Err(format!("Failed to write to output: {}", e)),
                        _ => (),
                    }

                    // Write the content of the element and any text that immediately follows it
                    self.parse_element(child, output, replacements)?;
                    match write!(output, "[/{}]", child.tag().name().to_uppercase()) {
                        Err(e) => return Err(format!("Failed to write to output: {}", e)),
                        _ => (),
                    }
                }
            }
            self.output_text(child.tail(), output, replacements)?;
        }

        Ok(())
    }

    /// Outputs a text string, formatting it and replacing template parameters as required.
    ///
    /// Newlines, indentation and extra spaces on the end of lines are deleted.
    fn output_text(&mut self,
                   text: &str,
                   output: &mut std::io::BufWriter<std::fs::File>,
                   replacements: &std::collections::HashMap<String, elementtree::Element>)
                   -> Result<(), String> {
        let replacements_re = regex::Regex::new(r"\{([\w-]+)\}").unwrap();
        let compact_text = Parser::compact_text(text, "");
        match replacements_re.captures(&compact_text) {
            Some(params) => {
                let normal_texts = replacements_re.split(&compact_text);
                let mut i = 1;
                for normal_text in normal_texts {
                    match output.write(normal_text.as_bytes()) {
                        Err(e) => return Err(format!("Failed to write to output: {}", e)),
                        _ => (),
                    }
                    match params.get(i) {
                        Some(param) => {
                            match replacements.get(param.as_str()) {
                                Some(replacement) => {
                                    self.parse_element(replacement,
                                                       output,
                                                       &std::collections::HashMap::new())?;
                                    ()
                                }
                                None => (),
                            }
                        }
                        None => (),
                    }
                    i += 1;
                }
            }
            None => {
                match output.write(compact_text.as_bytes()) {
                    Err(e) => return Err(format!("Failed to write to output: {}", e)),
                    _ => (),
                }
            }
        }
        Ok(())
    }

    /// Replaces all newlines (including preceding and succeeding whitespace) with a replacement.
    fn compact_text(text: &str, replacement: &str) -> String {
        let spaces_re = regex::Regex::new(r"(?:\s*(?:\r?\n)\s*)+").unwrap();
        spaces_re
            .replace_all(text, replacement)
            .into_owned()
            .to_owned()
    }
}
