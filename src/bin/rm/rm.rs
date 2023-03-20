use cargo_edit_9::CargoResult;
use cargo_edit_9::{colorize_stderr, manifest_from_pkgid, LocalManifest};
use clap::Args;
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

/// Remove a dependency from a Cargo.toml manifest file.
#[derive(Debug, Args)]
#[clap(version)]
pub struct RmArgs {
    /// Crates to be removed.
    #[clap(value_name = "CRATE", required = true)]
    crates: Vec<String>,

    /// Remove crate as development dependency.
    #[clap(long, short = 'D', conflicts_with = "build")]
    dev: bool,

    /// Remove crate as build dependency.
    #[clap(long, short = 'B', conflicts_with = "dev")]
    build: bool,

    /// Path to the manifest to remove a dependency from.
    #[clap(
        long,
        value_name = "PATH",
        parse(from_os_str),
        conflicts_with = "pkgid"
    )]
    manifest_path: Option<PathBuf>,

    /// Package id of the crate to remove this dependency from.
    #[clap(
        long = "package",
        short = 'p',
        value_name = "PKGID",
        conflicts_with = "manifest-path"
    )]
    pkgid: Option<String>,

    /// Unstable (nightly-only) flags
    #[clap(short = 'Z', value_name = "FLAG", global = true, arg_enum)]
    unstable_features: Vec<UnstableOptions>,

    /// Do not print any output in case of success.
    #[clap(long, short)]
    quiet: bool,
}

impl RmArgs {
    pub fn exec(&self) -> CargoResult<()> {
        exec(self)
    }

    /// Get depenency section
    pub fn get_section(&self) -> &'static str {
        if self.dev {
            "dev-dependencies"
        } else if self.build {
            "build-dependencies"
        } else {
            "dependencies"
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ArgEnum)]
enum UnstableOptions {}

fn print_msg(name: &str, section: &str) -> CargoResult<()> {
    let colorchoice = colorize_stderr();
    let mut output = StandardStream::stderr(colorchoice);
    output.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
    write!(output, "{:>12}", "Removing")?;
    output.reset()?;
    writeln!(output, " {} from {}", name, section)?;
    Ok(())
}

fn exec(args: &RmArgs) -> CargoResult<()> {
    let manifest_path = if let Some(ref pkgid) = args.pkgid {
        let pkg = manifest_from_pkgid(args.manifest_path.as_deref(), pkgid)?;
        Cow::Owned(Some(pkg.manifest_path.into_std_path_buf()))
    } else {
        Cow::Borrowed(&args.manifest_path)
    };
    let mut manifest = LocalManifest::find(manifest_path.as_deref())?;
    let deps = &args.crates;

    deps.iter()
        .map(|dep| {
            if !args.quiet {
                print_msg(dep, args.get_section())?;
            }
            let result = manifest
                .remove_from_table(args.get_section(), dep)
                .map_err(Into::into);

            // Now that we have removed the crate, if that was the last reference to that crate,
            // then we need to drop any explicitly activated features on that crate.
            manifest.gc_dep(dep);

            result
        })
        .collect::<CargoResult<Vec<_>>>()
        .map_err(|err| {
            eprintln!("Could not edit `Cargo.toml`.\n\nERROR: {}", err);
            err
        })?;

    manifest.write()?;

    Ok(())
}
