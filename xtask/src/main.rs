#![deny(missing_debug_implementations, rust_2018_idioms)]

pub mod build_docker_image;
pub mod hooks;
pub mod install_git_hooks;
pub mod target_info;
pub mod util;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use self::build_docker_image::BuildDockerImage;
use self::hooks::{Check, Test};
use self::install_git_hooks::InstallGitHooks;
use self::target_info::TargetInfo;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Toolchain name/version to use (such as stable or 1.59.0).
    #[clap(value_parser = is_toolchain)]
    toolchain: Option<String>,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract and print info for targets.
    TargetInfo(TargetInfo),
    /// Build a docker image from file.
    BuildDockerImage(BuildDockerImage),
    /// Install git development hooks.
    InstallGitHooks(InstallGitHooks),
    /// Run code formatting checks and lints.
    Check(Check),
    /// Run unittest suite.
    Test(Test),
}

fn is_toolchain(toolchain: &str) -> cross::Result<String> {
    if toolchain.starts_with('+') {
        Ok(toolchain.chars().skip(1).collect())
    } else {
        eyre::bail!("not a toolchain")
    }
}

pub fn main() -> cross::Result<()> {
    cross::install_panic_hook()?;
    let cli = Cli::parse();
    match cli.command {
        Commands::TargetInfo(args) => {
            let engine = get_container_engine(args.engine.as_deref())?;
            target_info::target_info(args, &engine)?;
        }
        Commands::BuildDockerImage(args) => {
            let engine = get_container_engine(args.engine.as_deref())?;
            build_docker_image::build_docker_image(args, &engine)?;
        }
        Commands::InstallGitHooks(args) => {
            install_git_hooks::install_git_hooks(args)?;
        }
        Commands::Check(args) => {
            hooks::check(args, cli.toolchain.as_deref())?;
        }
        Commands::Test(args) => {
            hooks::test(args, cli.toolchain.as_deref())?;
        }
    }

    Ok(())
}

fn get_container_engine(engine: Option<&str>) -> Result<PathBuf, which::Error> {
    if let Some(ce) = engine {
        which::which(ce)
    } else {
        cross::docker::get_container_engine()
    }
}
