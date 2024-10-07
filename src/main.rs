use std::env::current_dir;
use std::fs::remove_file;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::exit;

use clap::{Parser, Subcommand};
use path_dedot::ParseDot;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "breadcrumbs",
    version = "0.2.0",
    about = "Manage breadcrumb symlinks"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create breadcrumbs leading to the root directory from its subdirectories")]
    Scatter {
        #[arg(default_value = ".", help = "Directory to create breadcrumbs for")]
        root: PathBuf,

        #[arg(short, long, help = "Name of the breadcrumbs")]
        name: Option<String>,

        #[arg(
            short,
            long,
            help = "Maximum depth of subdirectories to create breadcrumbs for"
        )]
        max_depth: Option<usize>,
    },
    #[command(
        about = "Create breadcrumbs leading from one directory to another upwards in the hierarchy"
    )]
    Trail {
        #[arg(default_value = ".", help = "Directory the breadcrumb trail leads to")]
        to: PathBuf,

        #[arg(
            short,
            long,
            default_value = ".",
            help = "Directory the breadcrumb trail starts from"
        )]
        from: PathBuf,

        #[arg(short, long, help = "Name of the breadcrumbs")]
        name: Option<String>,
    },
    #[command(about = "Rename breadcrumbs")]
    Rename {
        #[arg(help = "Breadcrumb to rename")]
        breadcrumb: PathBuf,

        #[arg(short, long, help = "New name of the breadcrumbs")]
        name: String,
    },
    #[command(about = "Remove breadcrumbs")]
    Vacuum {
        #[arg(help = "Breadcrumb to vacuum")]
        breadcrumb: PathBuf,
    },
}

fn main() {
    let args = Args::parse();
    let result = match args.command {
        Commands::Scatter {
            root,
            name,
            max_depth,
        } => scatter(root, name, max_depth),
        Commands::Trail { to, from, name } => trail(to, from, name),
        Commands::Rename { breadcrumb, name } => rename(breadcrumb, name),
        Commands::Vacuum { breadcrumb } => vacuum(breadcrumb),
    };

    match result {
        Ok(stdout) => {
            println!("{stdout}");
        }
        Err(stderr) => {
            eprintln!("Error: {stderr}");
            exit(1);
        }
    }
}

fn scatter(
    root: PathBuf,
    name: Option<String>,
    max_depth: Option<usize>,
) -> Result<String, String> {
    let cwd = current_dir().map_err(|e| format!("could not get current working directory. {e}"))?;

    let root_path = &cwd
        .join(root)
        .parse_dot()
        .map_err(|e| format!("could not parse root path. {e}"))?
        .to_path_buf();

    let name = match name {
        Some(name) => name,
        None => root_path.file_name().unwrap().to_str().unwrap().to_string(),
    };
    let breadcrumb_name = format!("..{name}");

    let max_depth = max_depth.unwrap_or(usize::MAX);

    let root_breadcrumb_path = root_path.join(&breadcrumb_name);
    if !root_breadcrumb_path.exists() {
        symlink(".", &root_breadcrumb_path)
            .map_err(|e| format!("could not create root breadcrumb symlink. {e}"))?;
    }

    for entry in WalkDir::new(&root_path)
        .min_depth(1) // Skip the root directory itself
        .max_depth(max_depth)
    {
        let entry = entry.map_err(|e| format!("could not walk directory. {e}"))?;
        if !entry.file_type().is_dir() {
            continue;
        }

        let breadcrumb_path = entry.path().join(&breadcrumb_name);
        if breadcrumb_path.exists() {
            continue;
        }
        symlink(format!("../{breadcrumb_name}"), &breadcrumb_path)
            .map_err(|e| format!("could not create breadcrumb symlink. {e}"))?;
    }

    Ok(format!(
        "Scattered breadcrumbs.\n\
         Root: {root_path}\n\
         Name: {breadcrumb_name}",
        root_path = root_path.display(),
    ))
}

fn trail(to: PathBuf, from: PathBuf, name: Option<String>) -> Result<String, String> {
    let cwd = current_dir().map_err(|e| format!("could not get current working directory. {e}"))?;

    let to_path = &cwd
        .join(to)
        .parse_dot()
        .map_err(|e| format!("could not parse `to` path. {e}"))?
        .to_path_buf();
    let from_path = &cwd
        .join(from)
        .parse_dot()
        .map_err(|e| format!("could not parse `from` path. {e}"))?
        .to_path_buf();

    let name = match name {
        Some(name) => name,
        None => to_path.file_name().unwrap().to_str().unwrap().to_string(),
    };
    let breadcrumb_name = format!("..{name}");

    let trail_path = from_path.strip_prefix(&to_path).map_err(|_| {
        format!(
            "`to` path must be a subpath of `from` path.\n\
             From: {from_path}\n\
             To:   {to_path}",
            from_path = from_path.display(),
            to_path = to_path.display(),
        )
    })?;

    let root_breadcrumb_path = to_path.join(&breadcrumb_name);
    if !root_breadcrumb_path.exists() {
        symlink(".", &root_breadcrumb_path)
            .map_err(|e| format!("could not create root breadcrumb symlink. {e}"))?;
    }

    let mut trailing_path = to_path.clone();
    for trail_component in trail_path.components() {
        trailing_path = trailing_path.join(trail_component);
        let breadcrumb_path = trailing_path.join(&breadcrumb_name);
        if breadcrumb_path.exists() {
            continue;
        }
        symlink(format!("../{breadcrumb_name}"), &breadcrumb_path)
            .map_err(|e| format!("could not create breadcrumb symlink. {e}"))?;
    }

    Ok(format!(
        "Created trail of breadcrumbs.\n\
         From:   {from_path}\n\
         To:     {to_path}\n\
         Trail: ./{trail_path}\n\
         Name: {breadcrumb_name}",
        from_path = from_path.display(),
        to_path = to_path.display(),
        trail_path = trail_path.display(),
    ))
}

fn rename(breadcrumb: PathBuf, name: String) -> Result<String, String> {
    let old_breadcrumb_name = breadcrumb
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    if !old_breadcrumb_name.starts_with("..") {
        return Err("not a breadcrumb".to_string());
    }

    let new_breadcrumb_name = format!("..{name}");

    let root_path = breadcrumb
        .canonicalize()
        .map_err(|e| format!("could not read breadcrumb symlink. {e}"))?;

    let old_root_breadcrumb_path = root_path.join(&old_breadcrumb_name);
    let new_root_breadcrumb_path = old_root_breadcrumb_path.with_file_name(&new_breadcrumb_name);

    symlink(".", &new_root_breadcrumb_path)
        .map_err(|e| format!("could not create new root breadcrumb symlink. {e}"))?;
    remove_file(&old_root_breadcrumb_path)
        .map_err(|e| format!("could not remove old root breadcrumb symlink. {e}"))?;

    for entry in WalkDir::new(&root_path)
        .min_depth(1) // Skip the root directory itself
        .into_iter()
        .filter_entry(|e| e.path().join(&old_breadcrumb_name).is_symlink())
    {
        let old_breadcrumb_path = entry
            .map_err(|e| format!("could not walk directory. {e}"))?
            .path()
            .join(&old_breadcrumb_name);
        let new_breadcrumb_path = old_breadcrumb_path.with_file_name(&new_breadcrumb_name);

        remove_file(&old_breadcrumb_path)
            .map_err(|e| format!("could not remove old breadcrumb symlink. {e}"))?;
        symlink(format!("../{new_breadcrumb_name}"), &new_breadcrumb_path)
            .map_err(|e| format!("could not create new breadcrumb symlink. {e}"))?;
    }

    Ok(format!(
        "Renamed breadcrumbs.\n\
         Root: {root_path}\n\
         Old Name: {old_breadcrumb_name}\n\
         New Name: {new_breadcrumb_name}",
        root_path = root_path.display(),
    ))
}

fn vacuum(breadcrumb: PathBuf) -> Result<String, String> {
    let breadcrumb_name = breadcrumb
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    if !breadcrumb_name.starts_with("..") {
        return Err("not a breadcrumb".to_string());
    }

    let root_path = breadcrumb
        .canonicalize()
        .map_err(|e| format!("could not read breadcrumb symlink. {e}"))?;

    for entry in WalkDir::new(&root_path)
        .into_iter()
        .filter_entry(|e| e.path().join(&breadcrumb_name).is_symlink())
    {
        let breadcrumb_path = entry
            .map_err(|e| format!("could not walk directory. {e}"))?
            .path()
            .join(&breadcrumb_name);
        remove_file(&breadcrumb_path)
            .map_err(|e| format!("could not remove breadcrumb symlink. {e}"))?;
    }

    Ok(format!(
        "Vacuumed breadcrumbs.\n\
         Root: {root_path}\n\
         Name: {breadcrumb_name}",
        root_path = root_path.display(),
    ))
}
