//! Parsing of Inko source code into ASTs.
use crate::diagnostics::DiagnosticId;
use crate::state::{BuildTags, State};
use ast::nodes::{Module, Node, TopLevelExpression};
use ast::parser::Parser;
use ast::source_location::SourceLocation;
use std::collections::{HashMap, HashSet};
use std::fs::read;
use std::path::PathBuf;
use types::module_name::ModuleName;

fn imported_modules(
    module: &mut Module,
    tags: &BuildTags,
) -> Vec<(ModuleName, SourceLocation)> {
    let mut names = Vec::new();

    for expr in &mut module.expressions {
        let (path, loc) = match expr {
            TopLevelExpression::Import(ref mut node) => {
                node.include = node.tags.as_ref().map_or(true, |n| {
                    n.values.iter().all(|i| tags.is_defined(&i.name))
                });

                if !node.include {
                    continue;
                }

                (&node.path, node.location().clone())
            }
            _ => continue,
        };

        let name = ModuleName::from(
            path.steps.iter().map(|i| i.name.clone()).collect::<Vec<_>>(),
        );

        names.push((name, loc));
    }

    names
}

/// A parsed module and the modules it depends on.
pub(crate) struct ParsedModule {
    pub(crate) name: ModuleName,
    pub(crate) ast: Module,
}

/// A compiler pass for parsing all the modules into an AST.
pub(crate) struct ModulesParser<'a> {
    state: &'a mut State,
}

impl<'a> ModulesParser<'a> {
    pub(crate) fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    /// Parses an initial set of modules and all their dependencies.
    ///
    /// Modules are parsed in a depth-first order. That is, given these imports:
    ///
    ///     import foo
    ///     import bar
    ///
    /// We first parse the surrounding module, then `foo`, then `bar`.
    pub(crate) fn run(
        &mut self,
        initial: Vec<(ModuleName, PathBuf)>,
    ) -> Vec<ParsedModule> {
        let mut scheduled = HashSet::new();
        let mut modules = HashMap::new();
        let mut pending = initial;

        for (_, path) in &pending {
            scheduled.insert(path.clone());
        }

        for name in &self.state.config.implicit_imports {
            // Implicitly imported modules are always part of std, so we
            // don't need to search through all the source paths.
            let path = self.state.config.std.join(name.to_path());

            scheduled.insert(path.clone());
            pending.push((name.clone(), path));
        }

        while let Some((qname, file)) = pending.pop() {
            if let Some(mut ast) = self.parse(&file) {
                let deps = imported_modules(&mut ast, &self.state.build_tags);

                modules
                    .insert(qname.clone(), ParsedModule { name: qname, ast });

                for (dep, location) in deps {
                    let path = if let Some(val) =
                        self.state.module_path(file.clone(), &dep)
                    {
                        val
                    } else {
                        self.state.diagnostics.error(
                            DiagnosticId::InvalidFile,
                            format!("The module '{}' couldn't be found", dep),
                            file.clone(),
                            location,
                        );

                        continue;
                    };

                    if scheduled.contains(&path) {
                        continue;
                    }

                    scheduled.insert(path.clone());
                    pending.push((dep, path));
                }
            }
        }

        let mut result: Vec<ParsedModule> = modules.into_values().collect();

        // We sort the modules so we process them in a deterministic order,
        // resulting in diagnostics being produced in a deterministic order.
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    fn parse(&mut self, file: &PathBuf) -> Option<Module> {
        let input = match read(file) {
            Ok(result) => result,
            Err(err) => {
                self.state.diagnostics.error(
                    DiagnosticId::InvalidFile,
                    format!(
                        "Failed to read {:?}: {}",
                        file.to_string_lossy(),
                        err
                    ),
                    file.clone(),
                    SourceLocation::new(1..=1, 1..=1),
                );

                return None;
            }
        };

        let mut parser = Parser::new(input, file.clone());

        match parser.parse() {
            Ok(ast) => Some(ast),
            Err(err) => {
                self.state.diagnostics.error(
                    DiagnosticId::InvalidSyntax,
                    err.message,
                    file.clone(),
                    err.location,
                );

                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::env::temp_dir;
    use std::fs::{remove_file, write};

    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new(name: &str) -> Self {
            Self { path: temp_dir().join(format!("{}.inko", name)) }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = remove_file(&self.path);
        }
    }

    #[test]
    fn test_run_with_existing_modules() {
        let file1 = TempFile::new("parsing1a");
        let file2 = TempFile::new("parsing2a");

        write(file1.path(), "import parsing2a").unwrap();
        write(file2.path(), "let A = 10").unwrap();

        let mut state = State::new(Config::new());

        state.config.sources.push(temp_dir());
        state.config.implicit_imports = Vec::new();

        let mut pass = ModulesParser::new(&mut state);
        let mods = pass.run(vec![(ModuleName::main(), file1.path().clone())]);

        assert_eq!(mods.len(), 2);

        let names = mods.iter().map(|m| m.name.clone()).collect::<Vec<_>>();

        assert!(names.contains(&ModuleName::main()));
        assert!(names.contains(&ModuleName::new("parsing2a")));
        assert_eq!(state.diagnostics.iter().count(), 0);
    }

    #[test]
    fn test_run_with_syntax_error() {
        let file1 = TempFile::new("parsing1b");
        let file2 = TempFile::new("parsing2b");

        write(file1.path(), "import parsing2b").unwrap();
        write(file2.path(), "10").unwrap();

        let mut state = State::new(Config::new());

        state.config.sources.push(temp_dir());
        state.config.implicit_imports = Vec::new();

        let mut pass = ModulesParser::new(&mut state);
        let mods = pass.run(vec![(ModuleName::main(), file1.path().clone())]);

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, ModuleName::main());
        assert_eq!(state.diagnostics.iter().count(), 1);
    }

    #[test]
    fn test_run_with_missing_file() {
        let file1 = TempFile::new("parsing1c");

        write(file1.path(), "import parsing2c").unwrap();

        let mut state = State::new(Config::new());

        state.config.sources.push(temp_dir());
        state.config.implicit_imports = Vec::new();

        let mut pass = ModulesParser::new(&mut state);
        let mods = pass.run(vec![(ModuleName::main(), file1.path().clone())]);

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, ModuleName::main());
        assert_eq!(state.diagnostics.iter().count(), 1);
    }

    #[test]
    fn test_run_with_implicit_imports() {
        let file1 = TempFile::new("parsing1d");
        let file2 = TempFile::new("parsing2d");

        write(file1.path(), "").unwrap();
        write(file2.path(), "let A = 10").unwrap();

        let mut state = State::new(Config::new());

        state.config.std = temp_dir();
        state.config.implicit_imports = vec![ModuleName::new("parsing2d")];

        let mut pass = ModulesParser::new(&mut state);
        let mods = pass.run(vec![(ModuleName::main(), file1.path().clone())]);

        assert_eq!(mods.len(), 2);

        let names = mods.iter().map(|m| m.name.clone()).collect::<Vec<_>>();

        assert!(names.contains(&ModuleName::main()));
        assert!(names.contains(&ModuleName::new("parsing2d")));
        assert_eq!(state.diagnostics.iter().count(), 0);
    }
}
