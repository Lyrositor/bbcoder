# bbcoder

[![Build Status](https://travis-ci.org/Lyrositor/bbcoder.svg?branch=master)](https://travis-ci.org/Lyrositor/bbcoder)

*BBCode generation tool using BBXML as a markup language*

**bbcoder** is a command-line tool for generating BBCode from a BBXML project, a custom markup language based on XML.
It allows for the definition of templates, pseudo-classes and including other files, greatly simplifying complex BBCode projects.

## Building

bbcoder is programmed in the [Rust](https://www.rust-lang.org/) programming language.
As such, you can use Cargo to build the executable with the following command:

```bash
cargo build --release
```

The main bbcoder executable should then be available at `target/release/bbcoder` (or `target/release/bbcoder.exe` on Windows).

## Using

bbcoder works with BBXML projects - a folder containing a `project.xml` file at its root, along with a collection of library and target files.
The recommended directory setup is:
```
project_root
|-- include/
    |-- lib1
    |-- lib2
    `-- ...
|-- src/
    |-- lib/
        `-- ...
    |-- target1.xml
    |-- target2.xml
    `-- ...
`-- project.xml
```

The `include/` directory contains third-party libraries your project uses, while the `src/` directory contains project-specific files.
The project's targets should all consist of single files placed under `src/`, preferably with the same name as the target they build.
Files common to multiple targets in your project should be placed in `src/lib/`.

To run a target, you have to point bbcoder to the `project.xml` file (by default, it looks for it in the working directory) and specify the target's name (by default, it builds the default target in the project file):

```
bbcoder [OPTIONS] [TARGET]
```

### `project.xml`

TBD

### BBXML Files

TBD

## License

bbcoder is licensed under the [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) license.
You are free to use, modify and redistribute this code without attribution or additional restrictions.
