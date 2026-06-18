use std::fs;

use zed_extension_api::{self as zed, Result};

struct SuperHtmlExtension {
    cached_binary_path: Option<String>,
}

#[derive(Clone)]
struct SuperHtmlBinary(String);

impl SuperHtmlExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<SuperHtmlBinary> {
        if let Some(path) = worktree.which("superhtml") {
            return Ok(SuperHtmlBinary(path));
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(SuperHtmlBinary(path.clone()));
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "kristoff-it/superhtml",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_without_ext = format!(
            "{arch}-{os}",
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X8664 => "x86_64",
                zed::Architecture::X86 => return Err("unsupported architecture".into()),
            },
            os = match platform {
                zed::Os::Mac => "macos",
                zed::Os::Linux => "linux-musl",
                zed::Os::Windows => "windows",
            }
        );
        let asset_name = format!(
            "{asset_without_ext}.{ext}",
            ext = match platform {
                zed::Os::Linux => "tar.xz",
                zed::Os::Mac | zed::Os::Windows => "zip",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "no asset found matching {:?}, available assets: {:#?}",
                    asset_name, release.assets
                )
            })?;

        let version_dir = format!("superhtml-{}", release.version);

        let binary_dir = format!("{version_dir}/{asset_without_ext}");
        fs::create_dir_all(&binary_dir).map_err(|e| format!("failed to create directory: {e}"))?;
        let binary_path = format!(
            "{version_dir}/{asset_without_ext}/{binary}",
            binary = match platform {
                zed::Os::Mac | zed::Os::Linux => "superhtml",
                zed::Os::Windows => "superhtml.exe",
            }
        );

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &binary_path,
                match platform {
                    zed::Os::Linux => zed::DownloadedFileType::Uncompressed,
                    zed::Os::Mac | zed::Os::Windows => zed::DownloadedFileType::Zip,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            match platform {
                zed::Os::Linux => {
                    let absolute_exetension_dir = std::env::current_dir()
                        .map_err(|err| format!("can't get current dir: {err}"))?;
                    let output = zed::Command::new("tar")
                        .args([
                            "-xf",
                            &absolute_exetension_dir
                                .join(&binary_path)
                                .display()
                                .to_string(),
                        ])
                        .args([
                            "-C",
                            &absolute_exetension_dir
                                .join(&binary_dir)
                                .display()
                                .to_string(),
                        ])
                        .output()
                        .map_err(|err| {
                            format!("failed to extract language server, tar required: {err}")
                        })?;
                    if output.status.is_none_or(|status| status != 0) {
                        return Err(format!(
                            "failed to extract language server: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
                zed::Os::Mac | zed::Os::Windows => {
                    zed::make_file_executable(&binary_path)?;
                }
            }

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(SuperHtmlBinary(binary_path))
    }
}

impl zed::Extension for SuperHtmlExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let SuperHtmlBinary(path) = self.language_server_binary(language_server_id, worktree)?;
        Ok(zed::Command::new(path).arg("lsp"))
    }
}

zed::register_extension!(SuperHtmlExtension);
