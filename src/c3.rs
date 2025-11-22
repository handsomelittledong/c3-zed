use std::{
    fs::{self, File},
    io::{ErrorKind, Read},
};

use zed_extension_api::{
    self as zed, current_platform, download_file, latest_github_release, make_file_executable,
    Architecture, GithubReleaseOptions, Os, Result,
};

struct C3Extension;

impl C3Extension {
    fn download_lsp(release: &zed::GithubRelease) {
        {
            if let Ok(_) = match current_platform() {
                (Os::Windows, Architecture::X8664) => download_file(
                    &release.assets[2].download_url,
                    "c3lsp/",
                    zed_extension_api::DownloadedFileType::Zip,
                ),
                (Os::Mac, Architecture::Aarch64) => download_file(
                    &release.assets[0].download_url,
                    "c3lsp/",
                    zed_extension_api::DownloadedFileType::Zip,
                ),
                (Os::Linux, Architecture::X8664) => download_file(
                    &release.assets[1].download_url,
                    "c3lsp/",
                    zed_extension_api::DownloadedFileType::GzipTar,
                ),
                _ => Err("no available lsp!".to_string()),
            } {}
        }
    }
}

impl zed::Extension for C3Extension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if let Ok(release) = latest_github_release(
            "pherrymason/c3-lsp",
            GithubReleaseOptions {
                pre_release: false,
                require_assets: false,
            },
        ) {
            let mut file = match File::open("lsp_ver") {
                Ok(file_handle) => file_handle,
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => File::create("lsp_ver").unwrap(),
                    _ => return Err("Failed load file".to_string()),
                },
            };
            let mut content = String::new();

            file.read_to_string(&mut content).unwrap_or_default();

            if content != release.version {
                fs::write("lsp_ver", release.version.as_bytes())
                    .map_err(|_| "Failed to write file".to_string())?;
                Self::download_lsp(&release);
            }
        }

        let path = match current_platform() {
            (Os::Windows, Architecture::X8664) => "c3lsp/server/bin/release/c3lsp.exe",
            (Os::Mac, Architecture::Aarch64) => "c3lsp/server/bin/release/c3lsp",
            (Os::Linux, Architecture::X8664) => "c3lsp/server/bin/release/c3lsp",
            _ => "no available lsp!",
        };

        make_file_executable(&path)?;

        Ok(zed::Command {
            command: path.to_string(),
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(C3Extension);
