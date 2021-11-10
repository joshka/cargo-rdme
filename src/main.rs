/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#![cfg_attr(feature = "fatal-warnings", deny(warnings))]
#![deny(clippy::correctness)]
#![warn(clippy::pedantic)]
#![allow(clippy::match_bool)]
#![allow(clippy::if_not_else)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::use_self)]
#![allow(clippy::single_match_else)]
#![allow(clippy::inline_always)]
#![allow(clippy::partialeq_ne_impl)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::new_without_default)]
// Note: If you change this remember to update `README.md`.  To do so run `cargo run`.
//! # Cargo rdme
//!
//! Cargo command to create your README from your crate’s documentation.
//!
//! ## Usage
//!
//! Cargo rdme will insert your crate’s documentation in your README file.  To control where the
//! documentation will be inserted you need to insert a marker: `<!-- cargo-rdme -->`.  For example,
//! you can start your README with some glorious badges and follow up with the rustdoc
//! documentation:
//!
//! ```markdown
//! [![Build Status](https://example.org/badge.svg)](https://example.org/link-to-ci)
//!
//! <!-- cargo-rdme -->
//! ```
//!
//! After running `cargo rdme` you will find your README to be something like:
//!
//! ```markdown
//! [![Build Status](https://example.org/badge.svg)](https://example.org/link-to-ci)
//!
//! <!-- cargo-rdme start -->
//!
//! <WHATEVER-YOUR-CRATES-DOC-IS>
//!
//! <!-- cargo-rdme end -->
//! ```
//!
//! Whenever change your crate’s documentation you just need to run `cargo rdme` to update your
//! README file.
//!
//! ## Automatic transformations
//!
//! The documentation of your crate doesn’t always map directly to a good README.  For example,
//! rust code blocks can have hidden lines.  Those should not be shown in the README file.
//!
//! This section covers the transformation cargo rdme automatically apply to generate a better
//! README.
//!
//! ### Rust code block
//!
//! Rust code block are transformed in two ways by cargo rdme:
//!
//! 1. Rust code blocks with lines starting with `#` will be omitted, just like in `rustdoc`.
//! 2. Rust code blocks get annotated with the `rust` markdown tag so it gets proper syntax
//!    highlighting.  We also remove tags that only concern `rustdoc` such as `should_panic`.
//!
//! In the table below you can see an example of these modification.  The code block now is
//! tagged with `rust` and hidden lines were removed:
//!
//! <table border="1">
//! <col span="1" width="40%">
//! <col span="1" width="40%">
//! </colgroup>
//! <tr>
//! <th><center>Crate’s rustdoc</center></th>
//! <th><center>README.md</center></th>
//! <tr>
//! <tr>
//! <td>
//!
//! ```rust
//! //! To check if a number is prime do:
//! //!
//! //! ```
//! //! # fn main() {
//! //! for i in 2.. {
//! //!     if is_prime(i) {
//! //!         println!("{}", i);
//! //!     }
//! //! }
//! //! # }
//! //! ```
//! ```
//!
//! </td>
//! <td>
//!
//! ````markdown
//! To check if a number is prime do:
//!
//! ```rust
//! for i in 2.. {
//!     if is_prime(i) {
//!         println!("{}", i);
//!     }
//! }
//! ```
//! ````
//!
//! </td>
//! </tr>
//! </table>
//!
//! ### Intralinks
//!
//! Rust documentation can contain [links to items defined in the crate](https://doc.rust-lang.org/stable/rustdoc/linking-to-items-by-name.html).
//! This links would not make sense in your README file, so cargo rdme automatically generate
//! links to [docs.rs](https://docs.rs) for these intralinks.
//!
//! Currently we only support links of the form `[⋯](crate::⋯)`, so be sure to use that format.
//! Links to the standard library are also supported, and they must be of the form
//! `[⋯](::<crate>::⋯)`, where `<crate>` is a crate that is part of the standard library, such as
//! `std`, `core`, or `alloc`.
//!
//! Take a look at the example below:
//!
//! <table border="1">
//! <col span="1" width="40%">
//! <col span="1" width="40%">
//! </colgroup>
//! <tr>
//! <th><center>Crate’s rustdoc</center></th>
//! <th><center>README.md</center></th>
//! <tr>
//! <tr>
//! <td>
//!
//! ```rust
//! //! To check if a number is prime use
//! //! [`is_prime`](crate::is_prime).
//! ```
//!
//! </td>
//! <td>
//!
//! ```markdown
//! To check if a number is prime use
//! [`is_prime`](https://docs.rs/prime/latest/prime/fn.is_prime.html)
//! ```
//!
//! </td>
//! </tr>
//! </table>
//!
//! Note that there is some limitations in intralink support.  This is a complex feature: cargo rdme
//! needs to do some work to be able to create the link to docs.rs.  This is because the link
//! includes the kind of item the intralink points to, in the case of `is_prime` we need to discover
//! that is a function to generate a link that ends in `fn.is_prime.html`.  Therefore, intralink
//! support should be considered "best effort" (for instance, don’t expect items generated by macros
//! to be resolved).  If cargo rdme is unable to generate the link it will still generate the README
//! file, but a warning will be emitted.
//!
//! ## Configuration file
//!
//! If the default behavior of `cargo rdme` is not appropriate for your project you can crate a
//! configuration file `.cargo-rdme.toml` in the root of your project.  This is how that
//! configuration file can look like:
//!
//! ```toml
//! # What line terminator to use when updating the README file.  This can be "lf" or "crlf".
//! line-terminator = "lf"
//!
//! # The default entrypoint will be `src/lib.rs`.  You can change that in the `entrypoint` table.
//! [entrypoint]
//! # The entrypoint type can be "lib" or "bin".
//! type = "bin"
//! # When you set type to "bin" the entrypoint default to `src/main.rs`.  If you have binary targets
//! # specified in your cargo manifest you can select them by name with `bin-name`.
//! bin-name = "my-bin-name"
//! ```
//!
//! These setting can be overridden with command line flags.  Run `cargo rdme --help` for more
//! information.
//!
//! ## Integration with CI
//!
//! To verify that your README is up to date with your crate’s documentation you can run
//! `cargo rdme --check`.  The exit code will be `0` if the README is up to date, or `2` if it’s
//! not.

use crate::console::{print_error, print_warning};
use crate::options::{EntrypointOpt, LineTerminatorOpt};
use cargo_rdme::transform::IntralinkError;
use cargo_rdme::{
    extract_doc_from_source_file, infer_line_terminator, inject_doc_in_readme, LineTerminator,
    Project,
};
use cargo_rdme::{Doc, ProjectError, Readme};
use std::path::{Path, PathBuf};
use thiserror::Error;

mod console;
mod options;

const EXIT_CODE_ERROR: i32 = 1;
/// Exit code when we run in "check mode" and the README is not up to date.
const EXIT_CODE_CHECK: i32 = 2;
/// Exit code we don't update the README because we would overwrite uncommitted changes.
const EXIT_CODE_README_NOT_UPDATED_UNCOMMITTED_CHANGES: i32 = 3;

#[derive(Error, Debug)]
enum RunError {
    #[error("failed to get project info: {0}")]
    ProjectError(cargo_rdme::ProjectError),
    #[error("failed to extract rust doc: {0}")]
    ExtractDocError(cargo_rdme::ExtractDocError),
    #[error("failed to process README: {0}")]
    ReadmeError(cargo_rdme::ReadmeError),
    #[error("failed get crate's entry source file")]
    NoEntrySourceFile,
    #[error("failed get crate's README file")]
    NoReadmeFile,
    #[error("no crate-level rustdoc found")]
    NoRustdoc,
    #[error("failed to inject the documentation in the README: {0}")]
    InjectDocError(cargo_rdme::InjectDocError),
    #[error("IO error: {0}")]
    IOError(std::io::Error),
    #[error(
        "not updating README: it has uncommitted changes (use `--force` to bypass this check)"
    )]
    ReadmeNotUpdatedUncommittedChanges,
    #[error("failed to transform intralinks: {0}")]
    TransformIntraLinkError(IntralinkError),
}

impl From<cargo_rdme::ProjectError> for RunError {
    fn from(e: ProjectError) -> RunError {
        RunError::ProjectError(e)
    }
}

impl From<cargo_rdme::ExtractDocError> for RunError {
    fn from(e: cargo_rdme::ExtractDocError) -> RunError {
        RunError::ExtractDocError(e)
    }
}

impl From<cargo_rdme::ReadmeError> for RunError {
    fn from(e: cargo_rdme::ReadmeError) -> RunError {
        RunError::ReadmeError(e)
    }
}

impl From<cargo_rdme::InjectDocError> for RunError {
    fn from(e: cargo_rdme::InjectDocError) -> RunError {
        RunError::InjectDocError(e)
    }
}

impl From<std::io::Error> for RunError {
    fn from(e: std::io::Error) -> RunError {
        RunError::IOError(e)
    }
}

impl From<IntralinkError> for RunError {
    fn from(e: IntralinkError) -> RunError {
        RunError::TransformIntraLinkError(e)
    }
}

impl From<std::convert::Infallible> for RunError {
    fn from(_: std::convert::Infallible) -> RunError {
        unreachable!()
    }
}

/// Check if the README is up to date.
///
/// This will check if the README has the given line terminator as well.
fn is_readme_up_to_date(
    readme_path: impl AsRef<Path>,
    new_readme: &Readme,
    line_terminator: LineTerminator,
) -> Result<bool, RunError> {
    let current_readme_raw: String = std::fs::read_to_string(readme_path)?;
    let new_readme_raw: Vec<u8> = {
        let mut bytes: Vec<u8> = Vec::with_capacity(32 * 1024);
        new_readme.write(&mut bytes, line_terminator)?;
        bytes
    };

    Ok(current_readme_raw.as_bytes() == new_readme_raw.as_slice())
}

fn entrypoint(project: &Project, entrypoint_opt: EntrypointOpt) -> Option<&Path> {
    match entrypoint_opt {
        EntrypointOpt::Auto => {
            project.get_lib_entryfile_path().or_else(|| project.get_bin_default_entryfile_path())
        }
        EntrypointOpt::Lib => project.get_lib_entryfile_path(),
        EntrypointOpt::BinDefault => project.get_bin_default_entryfile_path(),
        EntrypointOpt::BinName(name) => project.get_bin_entryfile_path(&name),
    }
}

fn line_terminator(
    line_terminator_opt: LineTerminatorOpt,
    readme_path: impl AsRef<Path>,
) -> std::io::Result<LineTerminator> {
    match line_terminator_opt {
        LineTerminatorOpt::Auto => infer_line_terminator(readme_path),
        LineTerminatorOpt::Lf => Ok(LineTerminator::Lf),
        LineTerminatorOpt::CrLf => Ok(LineTerminator::CrLf),
    }
}

fn transform_doc(
    doc: &Doc,
    project: &Project,
    entrypoint: impl AsRef<Path>,
) -> Result<Doc, RunError> {
    use cargo_rdme::transform::{
        DocTransform, DocTransformIntralinks, DocTransformRustMarkdownTag,
        DocTransformRustRemoveComments,
    };

    let transform = DocTransformRustRemoveComments::new();
    // TODO Use `into_ok()` once it is stable (https://github.com/rust-lang/rust/issues/61695).
    let doc = transform.transform(doc)?;

    let transform = DocTransformRustMarkdownTag::new();
    // TODO Use `into_ok()` once it is stable (https://github.com/rust-lang/rust/issues/61695).
    let doc = transform.transform(&doc)?;

    let transform = DocTransformIntralinks::new(project.get_package_name(), entrypoint, |msg| {
        print_warning(msg);
    });

    Ok(transform.transform(&doc)?)
}

/// Check if the `path` has local changes that were not yet commited.
///
/// This returns `None` if we were not able to determine that.
fn git_is_current(path: impl AsRef<Path>) -> Option<bool> {
    use git2::{Repository, Status};

    let repository = Repository::discover(path.as_ref().parent()?).ok()?;
    let repository_path = repository.path().parent()?;

    let path_repository_base = path.as_ref().strip_prefix(repository_path).ok()?;

    let status = repository.status_file(path_repository_base).ok()?;

    Some(status == Status::CURRENT)
}

fn update_readme(
    new_readme: &Readme,
    readme_path: impl AsRef<Path>,
    line_terminator: LineTerminator,
    ignore_uncommitted_changes: bool,
) -> Result<(), RunError> {
    match ignore_uncommitted_changes || git_is_current(&readme_path).unwrap_or(true) {
        true => Ok(new_readme.write_to_file(&readme_path, line_terminator)?),
        false => Err(RunError::ReadmeNotUpdatedUncommittedChanges),
    }
}

fn run(options: options::Options) -> Result<(), RunError> {
    let project: Project = Project::from_current_dir()?;
    let entryfile: &Path =
        entrypoint(&project, options.entrypoint).ok_or(RunError::NoEntrySourceFile)?;
    let doc: Doc = match extract_doc_from_source_file(&entryfile)? {
        None => return Err(RunError::NoRustdoc),
        Some(doc) => doc,
    };

    let doc = transform_doc(&doc, &project, &entryfile)?;

    let readme_path: PathBuf = project.get_readme_path().ok_or(RunError::NoReadmeFile)?;
    let original_readme: Readme = Readme::from_file(&readme_path)?;
    let new_readme: Readme = inject_doc_in_readme(&original_readme, &doc)?;

    let line_terminator = line_terminator(options.line_terminator, &readme_path)?;

    match options.check {
        false => update_readme(&new_readme, readme_path, line_terminator, options.force),
        true => {
            if !is_readme_up_to_date(&readme_path, &new_readme, line_terminator)? {
                print_error("README is not up to date.");
                std::process::exit(EXIT_CODE_CHECK);
            }

            Ok(())
        }
    }
}

fn main() {
    let cmd_options = options::cmd_options();

    match std::env::current_dir() {
        Ok(current_dir) => {
            let config_file_options = match options::config_file_options(&current_dir) {
                Ok(opts) => opts,
                Err(e) => {
                    print_error(format!("unable to read config file: {}", e));
                    std::process::exit(EXIT_CODE_ERROR);
                }
            };

            let options = options::merge_options(cmd_options, config_file_options);

            if let Err(e) = run(options) {
                print_error(&e);

                let exit_code = match e {
                    RunError::ProjectError(_)
                    | RunError::ExtractDocError(_)
                    | RunError::ReadmeError(_)
                    | RunError::NoEntrySourceFile
                    | RunError::NoReadmeFile
                    | RunError::NoRustdoc
                    | RunError::InjectDocError(_)
                    | RunError::TransformIntraLinkError(_)
                    | RunError::IOError(_) => EXIT_CODE_ERROR,
                    RunError::ReadmeNotUpdatedUncommittedChanges => {
                        EXIT_CODE_README_NOT_UPDATED_UNCOMMITTED_CHANGES
                    }
                };

                std::process::exit(exit_code);
            }
        }
        Err(e) => {
            print_error(format!("unable to get current directory: {}", e));
            std::process::exit(EXIT_CODE_ERROR);
        }
    }
}
