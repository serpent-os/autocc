// SPDX-FileCopyrightText: Copyright © 2020-2024 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! autocc
//!
//! Simple helper to "do the right thing" and be a sensible `/usr/bin/cc` helper,
//! calling out to the right compiler (i.e. `/usr/bin/clang`) without needing mangling
//! of the filesystem

use std::{env, ffi::OsStr, io, os::unix::process::CommandExt, path::PathBuf, process};

/// Right now we only support GNU (gcc) and LLVM (clang)
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum Toolchain {
    // GNU (GCC)
    GNU(String),

    // LLVM (clang)
    LLVM(String),
}

impl AsRef<str> for Toolchain {
    fn as_ref(&self) -> &str {
        match self {
            Toolchain::GNU(s) => s,
            Toolchain::LLVM(s) => s,
        }
    }
}

/// Correctly demangle an environment variable into just the binary *name*
fn env_var_without_args(name: impl AsRef<OsStr>) -> Option<String> {
    let var = env::var(name.as_ref()).ok()?;

    let result = var.split('/').last()?.split(' ').next()?;
    Some(result.to_owned())
}

/// Attempt to find the tool relative to the path given (same dir)
fn tool_relative_to_path(path: impl AsRef<OsStr>, tool: &'static str) -> Option<String> {
    let path = PathBuf::from(path.as_ref());
    let input_path = path.parent()?;
    let tool_path = input_path.join(tool);
    if tool_path.exists() {
        Some(tool_path.to_str()?.to_owned())
    } else {
        None
    }
}

/// Try to return the correct toolchain based on the environment
fn toolchain_from_environment() -> Option<Toolchain> {
    // Query CC var
    if let Some(cc) = env_var_without_args("CC") {
        match cc.as_str() {
            "clang" => return Some(Toolchain::LLVM(env::var("CC").ok()?.to_owned())),
            "gcc" => return Some(Toolchain::GNU(env::var("CC").ok()?.to_owned())),
            x if x.contains("-gcc-") || x.ends_with("-gcc") => {
                return Some(Toolchain::GNU(env::var("CC").ok()?.to_owned()))
            }
            _ => {}
        }
    }

    // Query LD var
    if let Some(ld) = env_var_without_args("LD") {
        match ld.as_str() {
            "lld" => return Some(Toolchain::LLVM(tool_relative_to_path(&ld, "clang")?)),
            "ld" => return Some(Toolchain::GNU(tool_relative_to_path(&ld, "gcc")?)),
            x if x.starts_with("ld.") => {
                return Some(Toolchain::GNU(tool_relative_to_path(&ld, "gcc")?))
            }
            _ => {}
        }
    }

    None
}

fn find_in_path(name: impl AsRef<OsStr>) -> Option<String> {
    let path = env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".into());
    let name = name.as_ref();
    env::split_paths(&path)
        .filter_map(|p| {
            let tool_path = p.join(name);
            if tool_path.exists() {
                return Some(tool_path.to_string_lossy().to_string());
            } else {
                None
            }
        })
        .next()
}

/// Check well known filesystesm path
fn toolchain_from_filesystem() -> Option<Toolchain> {
    if let Some(clang) = find_in_path("clang") {
        Some(Toolchain::LLVM(clang))
    } else {
        find_in_path("gcc").map(Toolchain::GNU)
    }
}

/// Reexecute process as `cc` from whence we live, calling required toolchain
fn reexecute_with_args(compiler: &str) -> Result<(), io::Error> {
    let mut cmd = process::Command::new(compiler);
    cmd.arg0("/usr/bin/cc");
    cmd.args(env::args().skip(1));
    cmd.exec();

    eprintln!("cmd = {cmd:?}");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toolchain = if let Some(toolchain) = toolchain_from_environment() {
        Some(toolchain)
    } else {
        toolchain_from_filesystem()
    }
    .expect("failed to find compiler");

    reexecute_with_args(toolchain.as_ref())?;
    Ok(())
}
