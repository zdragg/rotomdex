// File managed by Codex
use std::{
    fs, io,
    num::NonZeroU32,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

use color_eyre::eyre::{Result, WrapErr, eyre};
use gix::{
    index::{State, entry::Flags},
    remote::{Direction, fetch::Shallow},
};
use prodash::tree::Item;

const URL: &str = "https://github.com/zdragg/rotomdex-data.git";
const SHALLOW: Shallow = Shallow::DepthAtRemote(NonZeroU32::MIN);

pub(super) fn download_repo(repo_path: &Path) -> Result<()> {
    let should_interrupt = AtomicBool::new(false);
    let root = prodash::tree::Root::new();
    let mut progress = root.add_child("downloading offline resources");

    let renderer = prodash::render::line::render(
        io::stdout(),
        Arc::downgrade(&root),
        prodash::render::line::Options {
            throughput: true,
            ..Default::default()
        }
        .auto_configure(prodash::render::line::StreamKind::Stderr),
    );

    let result = if repo_path.join(".git").exists() {
        update_repo(repo_path, &mut progress, &should_interrupt)
    } else {
        clone_repo(repo_path, &mut progress, &should_interrupt)
    };
    renderer.shutdown_and_wait();
    result
}

fn clone_repo(repo_path: &Path, progress: &mut Item, should_interrupt: &AtomicBool) -> Result<()> {
    let (mut checkout, _) = gix::prepare_clone(URL, repo_path)?
        .with_shallow(SHALLOW)
        .fetch_then_checkout(&mut *progress, should_interrupt)
        .wrap_err("Failed to download offline resources")?;
    let (_, outcome) = checkout.main_worktree(progress, should_interrupt)?;
    check_checkout(outcome)
}

fn update_repo(repo_path: &Path, progress: &mut Item, should_interrupt: &AtomicBool) -> Result<()> {
    let repo = gix::open(repo_path).wrap_err("Failed to open offline resource repository")?;
    let mut branch = repo
        .head_ref()?
        .ok_or_else(|| eyre!("Resource HEAD has no branch"))?;
    let upstream = branch
        .remote_ref_name(Direction::Fetch)
        .ok_or_else(|| eyre!("Resource branch has no upstream"))??;
    let fetched = branch
        .remote(Direction::Fetch)
        .ok_or_else(|| eyre!("Resource branch has no remote"))??
        .connect(Direction::Fetch)?
        .prepare_fetch(&mut *progress, Default::default())?
        .with_shallow(SHALLOW)
        .receive(&mut *progress, should_interrupt)
        .wrap_err("Failed to update offline resources")?;
    let target = fetched
        .ref_map
        .mappings
        .iter()
        .find(|mapping| mapping.remote.as_name() == Some(upstream.as_bstr()))
        .and_then(|mapping| mapping.remote.as_id())
        .ok_or_else(|| eyre!("Resource upstream was not advertised by the remote"))?;
    checkout_files(&repo, repo_path, target, progress, should_interrupt)?;
    branch.set_target_id(target.to_owned(), "update offline resources")?;
    Ok(())
}

fn checkout_files(
    repo: &gix::Repository,
    repo_path: &Path,
    target: &gix::hash::oid,
    progress: &mut Item,
    should_interrupt: &AtomicBool,
) -> Result<()> {
    let tree = repo.find_object(target)?.peel_to_tree()?;
    let old_index = repo.open_index()?;
    let mut index = repo.index_from_tree(&tree.id)?;
    remove_obsolete_files(repo_path, &old_index, &index)?;
    prepare_index(repo_path, &old_index, &mut index)?;
    let mut options =
        repo.checkout_options(gix::worktree::stack::state::attributes::Source::IdMapping)?;
    options.overwrite_existing = true;
    let outcome = gix::worktree::state::checkout(
        &mut index,
        repo_path,
        repo.objects.clone().into_arc()?,
        &progress.add_child("updating files"),
        &progress.add_child("writing bytes"),
        should_interrupt,
        options,
    )
    .wrap_err("Failed to check out updated resources")?;
    check_checkout(outcome)?;
    for entry in index.entries_mut() {
        entry.flags.remove(Flags::SKIP_WORKTREE);
    }
    index.write(Default::default())?;
    Ok(())
}

fn check_checkout(outcome: gix::worktree::state::checkout::Outcome) -> Result<()> {
    if !outcome.collisions.is_empty() || !outcome.errors.is_empty() {
        return Err(eyre!("Incomplete resource checkout: {outcome:?}"));
    }
    Ok(())
}

fn remove_obsolete_files(repo_path: &Path, old_index: &State, index: &State) -> Result<()> {
    // Checkout only writes entries; remove obsolete tracked files ourselves.
    for entry in old_index.entries() {
        let path = entry.path(old_index);
        if index.entry_by_path(path).is_some() {
            continue;
        }
        let path = repo_path.join(gix::path::try_from_bstr(path)?);
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) => {}
            Err(err) => {
                return Err(err).wrap_err_with(|| format!("Failed to remove {}", path.display()));
            }
        }
        for dir in path.ancestors().skip(1) {
            if dir == repo_path || fs::remove_dir(dir).is_err() {
                break;
            }
        }
    }

    Ok(())
}

fn prepare_index(repo_path: &Path, old_index: &State, index: &mut State) -> Result<()> {
    // Preserve unchanged files and their stat data instead of rewriting every resource.
    for (entry, path) in index.entries_mut_with_paths() {
        let path_on_disk = repo_path.join(gix::path::try_from_bstr(path)?);
        let Ok(metadata) = gix::index::fs::Metadata::from_path_no_follow(&path_on_disk) else {
            continue;
        };
        // Forced checkout would recursively delete a blocking directory.
        if metadata.is_dir() {
            fs::remove_dir(&path_on_disk).wrap_err_with(|| {
                format!("Directory blocks resource file {}", path_on_disk.display())
            })?;
        } else if let Some(old) = old_index.entry_by_path(path)
            && entry.id == old.id
            && entry.mode == old.mode
            && gix::index::entry::Stat::from_fs(&metadata)? == old.stat
        {
            entry.stat = old.stat;
            entry.flags.insert(Flags::SKIP_WORKTREE);
        }
    }
    Ok(())
}
