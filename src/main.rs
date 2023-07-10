use std::{collections::BTreeMap, path::PathBuf};

use clap::Parser;
use hash40::Hash40;
use motion_patch::MotionMapExt;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long)]
    pub source: PathBuf,

    #[clap(long)]
    pub target: PathBuf,

    #[clap(long)]
    pub output: PathBuf,

    #[clap(long)]
    pub labels: PathBuf,

    #[clap(long)]
    pub delete: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    Hash40::label_map()
        .lock()
        .unwrap()
        .add_labels_from_path(args.labels)?;

    let mut source_motion_lists = vec![];

    for entry in walkdir::WalkDir::new(&args.source) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        if entry.path().file_name().unwrap().to_str().unwrap() == "motion_list.bin" {
            source_motion_lists.push(entry.path().strip_prefix(&args.source)?.to_path_buf());
        }
    }

    for path in source_motion_lists {
        let costume = path.parent().unwrap();
        let motion = costume.parent().unwrap();
        let target_path = args.target.join(&path);
        if !target_path.exists() {
            continue;
        }

        let source = motion_lib::open(args.source.join(&path))?;
        let target = motion_lib::open(target_path)?;

        let patch = BTreeMap::create(&source, &target);

        let yaml = serde_yaml::to_string(&patch)?;
        let parent = args.output.join(motion);
        std::fs::create_dir_all(&parent)?;

        std::fs::write(parent.join("motion_patch.yaml"), &yaml)?;
    }

    if !args.delete {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(&args.target).contents_first(true) {
        let entry = entry?;

        if entry.file_type().is_dir() {
            if std::fs::read_dir(entry.path())?.next().is_none() {
                std::fs::remove_dir(entry.path())?;
            }
        } else {
            if entry.path().file_name().unwrap().to_str().unwrap() == "motion_list.bin" {
                std::fs::remove_file(entry.path())?;
            }
        }
    }

    Ok(())
}
