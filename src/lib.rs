use std::{env, fs, path::{Path, PathBuf}};

use zed_extension_api::{
    self as zed, Command, Extension, LanguageServerId, LanguageServerInstallationStatus, Result,
    Worktree, process::Command as ProcessCommand, register_extension, serde_json,
    set_language_server_installation_status, settings::LspSettings,
};

struct VyperExtension {
    cached_binary_path: Option<String>,
}

struct LaunchSettings {
    args: Vec<String>,
    env: Vec<(String, String)>,
    explicit_path: Option<String>,
}

impl VyperExtension {
    const LANGUAGE_SERVER_ID: &'static str = "vyper-lsp";
    const MANAGED_VERSION: &'static str = "0.1.4";

    fn lsp_settings(worktree: &Worktree) -> Option<LspSettings> {
        LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree).ok()
    }

    fn launch_settings(worktree: &Worktree) -> LaunchSettings {
        let mut env = worktree.shell_env();
        let mut args = Vec::new();
        let mut explicit_path = None;

        if let Some(settings) = Self::lsp_settings(worktree)
            && let Some(binary) = settings.binary
        {
            if let Some(binary_args) = binary.arguments {
                args = binary_args;
            }

            if let Some(binary_env) = binary.env {
                env.extend(binary_env);
            }

            explicit_path = binary.path;
        }

        LaunchSettings {
            args,
            env,
            explicit_path,
        }
    }

    fn env_var(env: &[(String, String)], key: &str) -> Option<String> {
        env.iter()
            .rev()
            .find_map(|(name, value)| (name == key && !value.is_empty()).then(|| value.clone()))
            .or_else(|| env::var(key).ok().filter(|value| !value.is_empty()))
    }

    fn managed_root_dir(env: &[(String, String)]) -> Result<PathBuf> {
        if let Some(pwd) = Self::env_var(env, "PWD") {
            return Ok(PathBuf::from(pwd).join(".zed-vyper"));
        }

        Ok(
            env::current_dir()
                .map_err(|err| format!("failed to determine a managed install directory: {err}"))?
                .join(".zed-vyper"),
        )
    }

    fn managed_version_dir(root_dir: &Path) -> PathBuf {
        root_dir.join(format!(
            "{}-{}",
            Self::LANGUAGE_SERVER_ID,
            Self::MANAGED_VERSION
        ))
    }

    fn managed_binary_path(root_dir: &Path) -> PathBuf {
        let binary = if matches!(zed::current_platform().0, zed::Os::Windows) {
            "Scripts/vyper-lsp.exe"
        } else {
            "bin/vyper-lsp"
        };

        Self::managed_version_dir(root_dir).join(binary)
    }

    fn managed_python_path(root_dir: &Path) -> PathBuf {
        let python = if matches!(zed::current_platform().0, zed::Os::Windows) {
            "Scripts/python.exe"
        } else {
            "bin/python"
        };

        Self::managed_version_dir(root_dir).join(python)
    }

    fn mark_status(
        language_server_id: &LanguageServerId,
        status: LanguageServerInstallationStatus,
    ) {
        set_language_server_installation_status(language_server_id, &status);
    }

    fn command_failed(message: &str, output: &zed::process::Output) -> String {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let status = output
            .status
            .map(|status| status.to_string())
            .unwrap_or_else(|| "terminated by signal".to_string());

        let mut details = format!("{message} (exit: {status})");

        if !stderr.is_empty() {
            details.push_str(&format!(": {stderr}"));
        } else if !stdout.is_empty() {
            details.push_str(&format!(": {stdout}"));
        }

        details
    }

    fn run_process(command: &mut ProcessCommand, message: &str) -> std::result::Result<(), String> {
        let output = command.output()?;
        if output.status == Some(0) {
            Ok(())
        } else {
            Err(Self::command_failed(message, &output))
        }
    }

    fn remove_outdated_versions(root_dir: &Path, current_version_dir: &Path) -> Result<()> {
        let entries = match fs::read_dir(root_dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => {
                return Err(format!(
                    "failed to list managed install directory {}: {err}",
                    root_dir.display()
                ));
            }
        };

        let current_name = current_version_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "failed to determine current managed install directory name".to_string())?;

        for entry in entries {
            let entry = entry.map_err(|err| {
                format!(
                    "failed to read managed install directory {}: {err}",
                    root_dir.display()
                )
            })?;
            let Some(name) = entry.file_name().to_str().map(|name| name.to_string()) else {
                continue;
            };

            if name.starts_with(Self::LANGUAGE_SERVER_ID) && name != current_name {
                fs::remove_dir_all(entry.path()).ok();
            }
        }

        Ok(())
    }

    fn ensure_uv_install(
        &mut self,
        language_server_id: &LanguageServerId,
        env: &[(String, String)],
    ) -> std::result::Result<String, String> {
        let root_dir = Self::managed_root_dir(env)?;
        let version_dir = Self::managed_version_dir(&root_dir);
        let binary_path = Self::managed_binary_path(&root_dir);
        let python_path = Self::managed_python_path(&root_dir);

        Self::mark_status(
            language_server_id,
            LanguageServerInstallationStatus::CheckingForUpdate,
        );

        Self::mark_status(
            language_server_id,
            LanguageServerInstallationStatus::Downloading,
        );

        fs::create_dir_all(&root_dir).map_err(|err| {
            format!(
                "failed to create managed install directory {}: {err}",
                root_dir.display()
            )
        })?;

        let mut venv = ProcessCommand::new("uv")
            .args([
                "venv",
                "--allow-existing",
                "--python",
                "3.12",
                &version_dir.to_string_lossy(),
            ])
            .envs(env.iter().cloned());
        Self::run_process(&mut venv, "failed to create managed uv environment")?;

        let mut install = ProcessCommand::new("uv")
            .args([
                "pip",
                "install",
                "--python",
                &python_path.to_string_lossy(),
                "--upgrade",
                &format!("vyper-lsp=={}", Self::MANAGED_VERSION),
            ])
            .envs(env.iter().cloned());
        Self::run_process(&mut install, "failed to install managed vyper-lsp with uv")?;

        Self::remove_outdated_versions(&root_dir, &version_dir)?;
        self.cached_binary_path = Some(binary_path.to_string_lossy().into_owned());
        Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
        Ok(binary_path.to_string_lossy().into_owned())
    }

    fn find_compatible_python(worktree: &Worktree, env: &[(String, String)]) -> Option<String> {
        for candidate in ["python3.12", "python3", "python"] {
            if worktree.which(candidate).is_none() {
                continue;
            }

            let mut version_check = ProcessCommand::new(candidate)
                .args([
                    "-c",
                    "import sys; raise SystemExit(0 if sys.version_info >= (3, 12) else 1)",
                ])
                .envs(env.iter().cloned());

            if version_check.output().ok().and_then(|output| output.status) == Some(0) {
                return Some(candidate.to_string());
            }
        }

        None
    }

    fn ensure_python_install(
        &mut self,
        language_server_id: &LanguageServerId,
        env: &[(String, String)],
        python_command: String,
    ) -> std::result::Result<String, String> {
        let root_dir = Self::managed_root_dir(env)?;
        let version_dir = Self::managed_version_dir(&root_dir);
        let binary_path = Self::managed_binary_path(&root_dir);
        let venv_python = Self::managed_python_path(&root_dir);

        Self::mark_status(
            language_server_id,
            LanguageServerInstallationStatus::CheckingForUpdate,
        );

        Self::mark_status(
            language_server_id,
            LanguageServerInstallationStatus::Downloading,
        );

        fs::create_dir_all(&root_dir).map_err(|err| {
            format!(
                "failed to create managed install directory {}: {err}",
                root_dir.display()
            )
        })?;

        let mut create_venv = ProcessCommand::new(python_command)
            .args(["-m", "venv", &version_dir.to_string_lossy()])
            .envs(env.iter().cloned());
        Self::run_process(
            &mut create_venv,
            "failed to create managed Python virtualenv",
        )?;

        let mut install = ProcessCommand::new(venv_python.to_string_lossy().into_owned())
            .args([
                "-m",
                "pip",
                "install",
                "--upgrade",
                "pip",
                &format!("vyper-lsp=={}", Self::MANAGED_VERSION),
            ])
            .envs(env.iter().cloned());
        Self::run_process(
            &mut install,
            "failed to install managed vyper-lsp with Python virtualenv",
        )?;

        Self::remove_outdated_versions(&root_dir, &version_dir)?;
        self.cached_binary_path = Some(binary_path.to_string_lossy().into_owned());
        Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
        Ok(binary_path.to_string_lossy().into_owned())
    }

    fn managed_or_system_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
        env: &[(String, String)],
    ) -> Result<String> {
        if let Some(cached_binary) = &self.cached_binary_path {
            Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
            return Ok(cached_binary.clone());
        }

        let mut errors = Vec::new();

        if worktree.which("uv").is_some() {
            match self.ensure_uv_install(language_server_id, env) {
                Ok(binary) => return Ok(binary),
                Err(err) => errors.push(err),
            }
        }

        if let Some(python_command) = Self::find_compatible_python(worktree, env) {
            match self.ensure_python_install(language_server_id, env, python_command) {
                Ok(binary) => return Ok(binary),
                Err(err) => errors.push(err),
            }
        }

        if let Some(system_binary) = worktree.which(Self::LANGUAGE_SERVER_ID) {
            Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
            return Ok(system_binary);
        }

        let mut message =
            "Unable to provision `vyper-lsp`. Tried managed install with `uv` and Python 3.12+, then checked $PATH for `vyper-lsp`.".to_string();

        if !errors.is_empty() {
            message.push_str("\n\n");
            message.push_str(&errors.join("\n"));
        } else {
            message.push_str(
                "\n\nInstall `uv`, a Python 3.12+ interpreter, or configure `lsp.vyper-lsp.binary.path`.",
            );
        }

        Self::mark_status(
            language_server_id,
            LanguageServerInstallationStatus::Failed(message.clone()),
        );
        Err(message)
    }
}

impl Extension for VyperExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let launch = Self::launch_settings(worktree);

        if let Some(explicit_path) = launch.explicit_path {
            Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
            return Ok(Command {
                command: explicit_path,
                args: launch.args,
                env: launch.env,
            });
        }

        let command = self.managed_or_system_binary(language_server_id, worktree, &launch.env)?;

        Ok(Command {
            command,
            args: launch.args,
            env: launch.env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options)
            .unwrap_or_default();

        Ok(Some(settings))
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings)
            .unwrap_or_default();

        Ok(Some(settings))
    }
}

register_extension!(VyperExtension);
