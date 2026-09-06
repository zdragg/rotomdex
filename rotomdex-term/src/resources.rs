use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Result, bail};
use git2::{AutotagOption, FetchOptions, ProxyOptions, RemoteCallbacks, Repository, ResetType, build::RepoBuilder};
use indicatif::{ProgressBar, ProgressStyle};

const URL: &str = "https://github.com/zdragg/rotomdex-data.git";

pub(crate) struct ResourcePaths {
    root: PathBuf,
}

impl ResourcePaths {
    pub(crate) fn new(data_dir: PathBuf) -> Self {
        Self {
            root: data_dir.join("resource"),
        }
    }

    pub(crate) fn resource_path(&self) -> PathBuf {
        self.root.clone()
    }

    pub(crate) fn download(&self) -> Result<()> {
        match Repository::open(&self.root) {
            Ok(repository) => update(&repository)?,
            Err(_) => {
                if self.root.exists() {
                    fs::remove_dir_all(&self.root)?;
                }
                fs::create_dir_all(self.root.parent().unwrap())?;
                clone(&self.root)?;
            }
        }

        self.validate()
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let repository = Repository::open(&self.root).wrap_err_with(|| {
            format!(
                "offline resources are missing or incomplete; run rotomdex --download ({})",
                self.root.display()
            )
        })?;

        if !repository.statuses(None)?.is_empty() {
            bail!("offline resources are incomplete or modified; run rotomdex --download");
        }

        for directory in ["api/v2", "sprites/pokemon"] {
            if !self.root.join(directory).is_dir() {
                bail!("offline resources are incomplete; run rotomdex --download");
            }
        }

        Ok(())
    }
}

fn clone(path: &Path) -> Result<()> {
    let progress = progress_bar();
    let result = RepoBuilder::new()
        .branch("main")
        .fetch_options(fetch_options(&progress))
        .clone(URL, path);
    progress.finish_and_clear();
    result.wrap_err("failed to download offline resources")?;
    Ok(())
}

fn update(repository: &Repository) -> Result<()> {
    let progress = progress_bar();
    let result = repository
        .find_remote("origin")?
        .fetch(&["main"], Some(&mut fetch_options(&progress)), None);
    progress.finish_and_clear();
    result.wrap_err("failed to update offline resources")?;

    let object = repository.revparse_single("FETCH_HEAD")?;
    repository.reset(&object, ResetType::Hard, None)?;
    Ok(())
}

fn fetch_options(progress: &ProgressBar) -> FetchOptions<'static> {
    let mut callbacks = RemoteCallbacks::new();
    let output = progress.clone();
    callbacks.sideband_progress(move |message| {
        output.suspend(|| eprint!("remote: {}", String::from_utf8_lossy(message)));
        true
    });

    let output = progress.clone();
    callbacks.transfer_progress(move |progress| {
        output.set_length(progress.total_objects() as u64);
        output.set_position(progress.received_objects() as u64);
        true
    });

    let mut proxy = ProxyOptions::new();
    proxy.auto();

    let mut options = FetchOptions::new();
    options
        .depth(1)
        .download_tags(AutotagOption::None)
        .proxy_options(proxy)
        .remote_callbacks(callbacks);
    options
}

fn progress_bar() -> ProgressBar {
    let progress = ProgressBar::new(0);
    progress.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} objects")
            .unwrap()
            .progress_chars("=>-"),
    );
    progress
}
