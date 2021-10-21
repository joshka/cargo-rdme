/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#![cfg_attr(feature = "fatal-warnings", deny(warnings))]
#![deny(clippy::correctness)]
#![warn(clippy::pedantic)]
#![allow(clippy::match_bool)]
#![allow(clippy::if_not_else)]
#![allow(clippy::stutter)]
#![allow(clippy::similar_names)]
#![allow(clippy::use_self)]
#![allow(clippy::single_match_else)]
#![allow(clippy::inline_always)]
#![allow(clippy::partialeq_ne_impl)]

use crate::markdown::{Markdown, MarkdownError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use toml::Value;

mod inject_doc;
mod markdown;

pub use inject_doc::{inject_doc, InjectDocError};

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("failed to read manifest \"{0}\"")]
    ErrorReadingManifest(PathBuf),
    #[error("failed to parse toml")]
    ErrorParsingToml,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Manifest {
    lib_path: Option<PathBuf>,
    readme_path: Option<PathBuf>,
    bin_path: HashMap<String, PathBuf>,
}

impl Manifest {
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Manifest, ManifestError> {
        let str: String = std::fs::read_to_string(&file_path)
            .map_err(|_| ManifestError::ErrorReadingManifest(file_path.as_ref().to_path_buf()))?;
        Manifest::from_str(&str)
    }

    pub fn from_str(str: &str) -> Result<Manifest, ManifestError> {
        let toml: toml::Value = toml::from_str(str).map_err(|_| ManifestError::ErrorParsingToml)?;

        let get_str = |value: &Value, field: &str| -> Option<String> {
            value.get(field).and_then(|p| p.as_str()).map(ToOwned::to_owned)
        };
        let get_str_table = |table: &str, field: &str| -> Option<&str> {
            toml.get(table).and_then(|v| v.get(field)).and_then(|p| p.as_str())
        };

        let mut bin_path = HashMap::new();

        if let Some(bin_table) = toml.get("bin").and_then(|v| v.as_array()) {
            for bin in bin_table {
                match (get_str(bin, "name"), get_str(bin, "path")) {
                    (Some(name), Some(path)) => {
                        bin_path.insert(name, Path::new(&path).to_path_buf());
                    }
                    _ => (),
                }
            }
        }

        toml.get("bin").and_then(|v| v.as_array()).map(|t| t.iter());

        Ok(Manifest {
            lib_path: get_str_table("lib", "path").map(|v| Path::new(v).to_path_buf()),
            readme_path: get_str_table("package", "readme").map(|v| Path::new(v).to_path_buf()),
            bin_path,
        })
    }
}

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("project root not found")]
    ProjectRootNotFound,
    #[error("manifest error: {0}")]
    ManifestError(ManifestError),
}

impl From<ManifestError> for ProjectError {
    fn from(e: ManifestError) -> ProjectError {
        ProjectError::ManifestError(e)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Project {
    manifest: Manifest,
    directory: PathBuf,
}

pub fn find_first_file_in_ancestors(dir_path: impl AsRef<Path>, filename: &str) -> Option<PathBuf> {
    for ancestor_dir in dir_path.as_ref().ancestors() {
        let file = ancestor_dir.join(filename);
        if file.is_file() {
            return Some(file);
        }
    }

    None
}

impl Project {
    /// Creates a [`Project`] from a path.  It will ancestor paths until it finds the root of the
    /// project.
    pub fn from_dir(dir_path: impl AsRef<Path>) -> Result<Project, ProjectError> {
        match find_first_file_in_ancestors(dir_path, "Cargo.toml") {
            None => Err(ProjectError::ProjectRootNotFound),
            Some(manifest_file) => Ok(Project {
                manifest: Manifest::from_file(&manifest_file)?,
                directory: manifest_file.parent().expect("this should never happen").to_path_buf(),
            }),
        }
    }

    pub fn get_lib_entryfile_path(&self) -> Option<PathBuf> {
        let default = || Path::new("src").join("lib.rs").to_path_buf();
        let rel_path = self.manifest.lib_path.clone().unwrap_or_else(default);
        let path = self.directory.join(rel_path).to_path_buf();

        match path.is_file() {
            true => Some(path),
            false => None,
        }
    }

    pub fn get_bin_default_entryfile_path(&self) -> Option<PathBuf> {
        let default = || Path::new("src").join("main.rs").to_path_buf();
        let rel_path = self.manifest.lib_path.clone().unwrap_or_else(default);
        let path = self.directory.join(rel_path).to_path_buf();

        match path.is_file() {
            true => Some(path),
            false => None,
        }
    }

    pub fn get_bin_entryfile_path(&self, name: &str) -> Option<PathBuf> {
        self.manifest.bin_path.get(name).and_then(|rel_path| {
            let path = self.directory.join(rel_path).to_path_buf();

            match path.is_file() {
                true => Some(path),
                false => None,
            }
        })
    }

    pub fn get_readme_path(&self) -> Option<PathBuf> {
        let default = || Path::new("README.md").to_path_buf();
        let rel_path = self.manifest.readme_path.clone().unwrap_or_else(default);
        let path = self.directory.join(rel_path).to_path_buf();

        match path.is_file() {
            true => Some(path),
            false => None,
        }
    }
}

#[derive(Error, Debug)]
pub enum DocError {
    #[error("cannot open source file \"{0}\"")]
    ErrorReadingSourceFile(PathBuf),
    #[error("cannot parse source file: {0}")]
    ErrorParsingSourceFile(syn::Error),
}

pub struct Doc {
    markdown: Markdown,
}

impl Doc {
    pub fn from_source_file(file_path: impl AsRef<Path>) -> Result<Option<Doc>, DocError> {
        let source: String = std::fs::read_to_string(file_path.as_ref())
            .map_err(|_| DocError::ErrorReadingSourceFile(file_path.as_ref().to_path_buf()))?;

        Doc::from_source_str(&source)
    }

    pub fn from_str(str: impl Into<String>) -> Doc {
        Doc { markdown: Markdown::from_str(str) }
    }

    fn is_toplevel_doc(attr: &syn::Attribute) -> bool {
        use syn::token::Bang;
        use syn::AttrStyle;

        attr.style == AttrStyle::Inner(Bang::default()) && attr.path.is_ident("doc")
    }

    pub fn from_source_str(source: &str) -> Result<Option<Doc>, DocError> {
        use syn::{parse_str, Lit, Meta, MetaNameValue};

        let ast: syn::File = parse_str(source).map_err(|e| DocError::ErrorParsingSourceFile(e))?;
        let mut lines: Vec<String> = Vec::with_capacity(1024);

        for attr in ast.attrs.iter() {
            if Doc::is_toplevel_doc(attr) {
                if let Ok(Meta::NameValue(MetaNameValue { lit: Lit::Str(lstr), .. })) =
                    attr.parse_meta()
                {
                    let string = &lstr.value();

                    match string.lines().count() {
                        0 => lines.push("".to_owned()),
                        1 => {
                            let line = string.strip_prefix(' ').unwrap_or(string);
                            lines.push(line.to_owned());
                        }

                        // Multiline comment.
                        _ => {
                            fn empty_line(str: &str) -> bool {
                                str.chars().all(|c| c.is_whitespace())
                            }

                            let x = string
                                .lines()
                                .enumerate()
                                .filter(|(i, l)| !(*i == 0 && empty_line(l)))
                                .map(|(_, l)| l);

                            lines.extend(x.map(|s| s.to_owned()));
                        }
                    }
                }
            }
        }

        match lines.is_empty() {
            true => Ok(None),
            false => Ok(Some(Doc { markdown: Markdown::from_lines(&lines) })),
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.markdown.lines()
    }
}

#[derive(Error, Debug)]
pub enum ReadmeError {
    #[error("failed to read README file \"{0}\"")]
    ErrorReadingReadmeFromFile(PathBuf),
    #[error("failed to write README file \"{0}\"")]
    ErrorWritingMarkdownToFile(PathBuf),
    #[error("failed to write README")]
    ErrorWritingMarkdown,
}

impl From<MarkdownError> for ReadmeError {
    fn from(e: MarkdownError) -> ReadmeError {
        match e {
            MarkdownError::ErrorReadingMarkdownFromFile(p) => {
                ReadmeError::ErrorReadingReadmeFromFile(p)
            }
            MarkdownError::ErrorWritingMarkdownToFile(p) => {
                ReadmeError::ErrorWritingMarkdownToFile(p)
            }
            MarkdownError::ErrorWritingMarkdown => ReadmeError::ErrorWritingMarkdown,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum LineTerminator {
    Lf,
    CrLf,
}

pub struct Readme {
    markdown: Markdown,
}

impl Readme {
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Readme, ReadmeError> {
        Ok(Readme { markdown: Markdown::from_file(file_path)? })
    }

    pub fn from_str(str: impl Into<String>) -> Readme {
        Readme { markdown: Markdown::from_str(str) }
    }

    pub fn from_lines(lines: &[impl AsRef<str>]) -> Readme {
        Readme { markdown: Markdown::from_lines(lines) }
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.markdown.lines()
    }

    pub fn write_to_file(
        &self,
        file: impl AsRef<Path>,
        line_terminator: LineTerminator,
    ) -> Result<(), ReadmeError> {
        Ok(self.markdown.write_to_file(file, line_terminator)?)
    }

    pub fn write(
        &self,
        writer: impl std::io::Write,
        line_terminator: LineTerminator,
    ) -> Result<(), ReadmeError> {
        Ok(self.markdown.write(writer, line_terminator)?)
    }
}

pub fn infer_line_terminator(file_path: impl AsRef<Path>) -> std::io::Result<LineTerminator> {
    let content: String = std::fs::read_to_string(file_path.as_ref())?;

    let crlf_lines: usize = content.matches("\r\n").count();
    let lf_lines: usize = content.matches("\n").count() - crlf_lines;

    if crlf_lines > lf_lines {
        Ok(LineTerminator::CrLf)
    } else {
        Ok(LineTerminator::Lf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::iter::FromIterator;

    #[test]
    fn test_manifest_from_str() {
        let str = indoc! { r#"
            [package]
            readme = "README.md"

            [lib]
            path = "src/lib.rs"
            "#
        };

        let expected_manifest = Manifest {
            lib_path: Some(Path::new("src").join("lib.rs").to_path_buf()),
            readme_path: Some(Path::new("README.md").to_path_buf()),
            bin_path: HashMap::new(),
        };

        assert_eq!(Manifest::from_str(str).unwrap(), expected_manifest);
    }

    #[test]
    fn test_manifest_from_str_multiple_bin() {
        let str = indoc! { r#"
            [package]

            [[bin]]
            name = "foo"
            path = "src/m.rs"

            [[bin]]
            name = "bar"
            path = "src/bar.rs"
            "#
        };

        let expected_manifest = Manifest {
            lib_path: None,
            readme_path: None,
            bin_path: HashMap::from_iter(
                [
                    ("foo".to_owned(), Path::new("src").join("m.rs")),
                    ("bar".to_owned(), Path::new("src").join("bar.rs")),
                ]
                .into_iter(),
            ),
        };

        assert_eq!(Manifest::from_str(str).unwrap(), expected_manifest);
    }

    #[test]
    fn test_doc_from_source_str_no_doc() {
        let str = indoc! { r#"
            use std::fs;

            struct Nothing {}
            "#
        };

        assert!(Doc::from_source_str(str).unwrap().is_none());
    }

    #[test]
    fn test_doc_from_source_str_single_line_comment() {
        let str = indoc! { r#"
            #![cfg_attr(not(feature = "std"), no_std)]
            // normal comment

            //! This is the doc for the crate.
            //!This line doesn't start with space.
            //!
            //! And a nice empty line above us.
            //! Also a line ending in "

            struct Nothing {}
            "#
        };

        let doc = Doc::from_source_str(str).unwrap().unwrap();
        let lines: Vec<&str> = doc.lines().collect();

        let expected = vec![
            "This is the doc for the crate.",
            "This line doesn't start with space.",
            "",
            "And a nice empty line above us.",
            "Also a line ending in \"",
        ];

        assert_eq!(lines, expected);
    }

    #[test]
    fn test_doc_from_source_str_multi_line_comment() {
        let str = indoc! { r#"
            #![cfg_attr(not(feature = "std"), no_std)]
            /* normal comment */

            /*!
            This is the doc for the crate.
             This line start with space.

            And a nice empty line above us.
            */

            struct Nothing {}
            "#
        };

        let doc = Doc::from_source_str(&str).unwrap().unwrap();
        let lines: Vec<&str> = doc.lines().collect();

        let expected = vec![
            "This is the doc for the crate.",
            " This line start with space.",
            "",
            "And a nice empty line above us.",
        ];

        assert_eq!(lines, expected);
    }

    #[test]
    fn test_doc_from_source_str_single_line_keep_indentation() {
        let str = indoc! { r#"
            #![cfg_attr(not(feature = "std"), no_std)]
            // normal comment

            //! This is the doc for the crate.  This crate does:
            //!
            //!   1. nothing.
            //!   2. niente.

            struct Nothing {}
            "#
        };

        let doc = Doc::from_source_str(str).unwrap().unwrap();
        let lines: Vec<&str> = doc.lines().collect();

        let expected = vec![
            "This is the doc for the crate.  This crate does:",
            "",
            "  1. nothing.",
            "  2. niente.",
        ];

        assert_eq!(lines, expected);
    }
}
