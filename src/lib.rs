use std::path::{Path, PathBuf};
use zed_extension_api::{
    self as zed, Command, Extension, LanguageServerId, LanguageServerInstallationStatus, Result,
    Worktree, register_extension, serde_json, set_language_server_installation_status,
    settings::LspSettings,
};

struct VyperExtension;

struct LaunchSettings {
    args: Vec<String>,
    env: Vec<(String, String)>,
    env_overrides: Vec<(String, String)>,
    explicit_path: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct WorkspaceVenv {
    bin_dirs: Vec<String>,
    site_packages: Vec<String>,
}

impl VyperExtension {
    const LANGUAGE_SERVER_ID: &'static str = "vyper-lsp";
    const MAX_VENV_ANCESTOR_STEPS: usize = 6;
    const FALLBACK_PYTHON_MINORS: [&'static str; 6] =
        ["3.15", "3.14", "3.13", "3.12", "3.11", "3.10"];

    fn lsp_settings(worktree: &Worktree) -> Option<LspSettings> {
        LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree).ok()
    }

    fn launch_settings(worktree: &Worktree) -> LaunchSettings {
        let env = worktree.shell_env();
        let mut args = Vec::new();
        let mut env_overrides = Vec::new();
        let mut explicit_path = None;

        if let Some(settings) = Self::lsp_settings(worktree)
            && let Some(binary) = settings.binary
        {
            if let Some(binary_args) = binary.arguments {
                args = binary_args;
            }

            if let Some(binary_env) = binary.env {
                env_overrides = binary_env.into_iter().collect();
            }

            explicit_path = binary.path;
        }

        LaunchSettings {
            args,
            env,
            env_overrides,
            explicit_path,
        }
    }

    fn mark_status(
        language_server_id: &LanguageServerId,
        status: LanguageServerInstallationStatus,
    ) {
        set_language_server_installation_status(language_server_id, &status);
    }

    fn missing_binary_message() -> String {
        "Unable to find `vyper-lsp`. Install it with `uv tool install vyper-lsp`, ensure it is available on your PATH, or configure `lsp.vyper-lsp.binary.path`.".to_string()
    }

    fn is_windows() -> bool {
        matches!(zed::current_platform().0, zed::Os::Windows)
    }

    fn path_list_separator() -> &'static str {
        if Self::is_windows() { ";" } else { ":" }
    }

    fn normalized_worktree_base(root_path: &str) -> PathBuf {
        let path = PathBuf::from(root_path);
        let looks_like_file = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "vy" | "vyi"));

        if looks_like_file {
            path.parent().map(Path::to_path_buf).unwrap_or(path)
        } else {
            path
        }
    }

    fn fallback_site_packages(venv_dir: &Path) -> Vec<String> {
        Self::FALLBACK_PYTHON_MINORS
            .iter()
            .map(|version| {
                venv_dir
                    .join("lib")
                    .join(format!("python{version}"))
                    .join("site-packages")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }

    fn push_unique(paths: &mut Vec<String>, path: String) {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }

    fn is_filesystem_root(path: &Path) -> bool {
        path.has_root() && path.parent().is_none()
    }

    fn ancestor_dirs(base: &Path) -> Vec<PathBuf> {
        let mut current = base.to_path_buf();
        let mut result = Vec::new();

        for _ in 0..=Self::MAX_VENV_ANCESTOR_STEPS {
            if current.as_os_str().is_empty() || Self::is_filesystem_root(&current) {
                break;
            }

            result.push(current.clone());

            let Some(parent) = current.parent().map(Path::to_path_buf) else {
                break;
            };
            if parent == current || parent.as_os_str().is_empty() {
                break;
            }

            current = parent;
        }

        result
    }

    fn workspace_venv_for_base_with_platform(base: &Path, is_windows: bool) -> WorkspaceVenv {
        let mut bin_dirs = Vec::new();
        let mut site_packages = Vec::new();

        for ancestor in Self::ancestor_dirs(base) {
            let venv_dir = ancestor.join(".venv");

            if is_windows {
                Self::push_unique(
                    &mut bin_dirs,
                    venv_dir.join("Scripts").to_string_lossy().into_owned(),
                );
                Self::push_unique(
                    &mut site_packages,
                    venv_dir
                        .join("Lib")
                        .join("site-packages")
                        .to_string_lossy()
                        .into_owned(),
                );
                continue;
            }

            Self::push_unique(
                &mut bin_dirs,
                venv_dir.join("bin").to_string_lossy().into_owned(),
            );

            for site_package in Self::fallback_site_packages(&venv_dir) {
                Self::push_unique(&mut site_packages, site_package);
            }
        }

        WorkspaceVenv {
            bin_dirs,
            site_packages,
        }
    }

    fn workspace_venv_for_base(base: &Path) -> WorkspaceVenv {
        Self::workspace_venv_for_base_with_platform(base, Self::is_windows())
    }

    fn workspace_venv(worktree: &Worktree) -> WorkspaceVenv {
        let base = Self::normalized_worktree_base(&worktree.root_path());
        Self::workspace_venv_for_base(&base)
    }

    fn set_env_var(env: &mut Vec<(String, String)>, key: &str, value: String) {
        env.retain(|(existing_key, _)| existing_key != key);
        env.push((key.to_string(), value));
    }

    fn prepend_env_path(env: &mut Vec<(String, String)>, key: &str, prefix: &str, separator: &str) {
        let mut segments = env
            .iter()
            .rev()
            .find_map(|(existing_key, value)| (existing_key == key).then(|| value.clone()))
            .unwrap_or_default()
            .split(separator)
            .filter(|segment| !segment.is_empty() && *segment != prefix)
            .map(str::to_string)
            .collect::<Vec<_>>();

        segments.insert(0, prefix.to_string());
        Self::set_env_var(env, key, segments.join(separator));
    }

    fn inject_workspace_venv(worktree: &Worktree, env: &mut Vec<(String, String)>) {
        let venv = Self::workspace_venv(worktree);
        let separator = Self::path_list_separator();

        // Make workspace-installed Vyper libraries visible to the global language server.
        for site_packages in venv.site_packages.iter().rev() {
            Self::prepend_env_path(env, "PYTHONPATH", site_packages, separator);
        }
        for bin_dir in venv.bin_dirs.iter().rev() {
            Self::prepend_env_path(env, "PATH", bin_dir, separator);
        }
    }

    fn apply_env_overrides(env: &mut Vec<(String, String)>, overrides: Vec<(String, String)>) {
        for (key, value) in overrides {
            Self::set_env_var(env, &key, value);
        }
    }

    fn ready_command(
        language_server_id: &LanguageServerId,
        command: String,
        args: Vec<String>,
        env: Vec<(String, String)>,
    ) -> Result<Command> {
        Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
        Ok(Command { command, args, env })
    }

    fn initialization_options_from_settings(
        settings: Option<LspSettings>,
    ) -> Option<serde_json::Value> {
        settings.and_then(|lsp_settings| lsp_settings.initialization_options)
    }

    fn workspace_configuration_from_settings(
        settings: Option<LspSettings>,
    ) -> Option<serde_json::Value> {
        settings.and_then(|lsp_settings| lsp_settings.settings)
    }
}

impl Extension for VyperExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let LaunchSettings {
            args,
            mut env,
            env_overrides,
            explicit_path,
        } = Self::launch_settings(worktree);

        Self::inject_workspace_venv(worktree, &mut env);
        Self::apply_env_overrides(&mut env, env_overrides);

        if let Some(explicit_path) = explicit_path {
            return Self::ready_command(language_server_id, explicit_path, args, env);
        }

        if let Some(path_binary) = worktree.which(Self::LANGUAGE_SERVER_ID) {
            return Self::ready_command(language_server_id, path_binary, args, env);
        }

        let message = Self::missing_binary_message();
        Self::mark_status(
            language_server_id,
            LanguageServerInstallationStatus::Failed(message.clone()),
        );
        Err(message)
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(Self::initialization_options_from_settings(
            LspSettings::for_worktree(language_server_id.as_ref(), worktree).ok(),
        ))
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(Self::workspace_configuration_from_settings(
            LspSettings::for_worktree(language_server_id.as_ref(), worktree).ok(),
        ))
    }
}

register_extension!(VyperExtension);

#[cfg(test)]
mod tests {
    use super::VyperExtension;
    use std::path::{Path, PathBuf};

    #[test]
    fn normalizes_vyper_file_root_to_parent() {
        let base = VyperExtension::normalized_worktree_base("/workspace/contracts/main/Foo.vy");
        assert_eq!(base, PathBuf::from("/workspace/contracts/main"));
    }

    #[test]
    fn generates_fallback_site_packages_candidates() {
        let venv_dir = Path::new("/workspace/.venv");
        let candidates = VyperExtension::fallback_site_packages(venv_dir);
        assert_eq!(
            candidates,
            vec![
                "/workspace/.venv/lib/python3.15/site-packages",
                "/workspace/.venv/lib/python3.14/site-packages",
                "/workspace/.venv/lib/python3.13/site-packages",
                "/workspace/.venv/lib/python3.12/site-packages",
                "/workspace/.venv/lib/python3.11/site-packages",
                "/workspace/.venv/lib/python3.10/site-packages",
            ]
        );
    }

    #[test]
    fn generates_unix_workspace_venv_candidates_for_all_ancestors() {
        let base = Path::new("/workspace/contracts/main");
        let venv = VyperExtension::workspace_venv_for_base_with_platform(base, false);
        assert_eq!(
            venv.bin_dirs,
            vec![
                "/workspace/contracts/main/.venv/bin",
                "/workspace/contracts/.venv/bin",
                "/workspace/.venv/bin",
            ]
        );

        assert_eq!(
            &venv.site_packages[..6],
            &[
                "/workspace/contracts/main/.venv/lib/python3.15/site-packages",
                "/workspace/contracts/main/.venv/lib/python3.14/site-packages",
                "/workspace/contracts/main/.venv/lib/python3.13/site-packages",
                "/workspace/contracts/main/.venv/lib/python3.12/site-packages",
                "/workspace/contracts/main/.venv/lib/python3.11/site-packages",
                "/workspace/contracts/main/.venv/lib/python3.10/site-packages",
            ]
        );
    }

    #[test]
    fn generates_windows_workspace_venv_candidates() {
        let venv = VyperExtension::workspace_venv_for_base_with_platform(Path::new("repo"), true);

        assert_eq!(venv.bin_dirs, vec!["repo/.venv/Scripts"]);
        assert_eq!(venv.site_packages, vec!["repo/.venv/Lib/site-packages"]);
    }

    #[test]
    fn initialization_options_are_none_when_unset() {
        assert_eq!(
            VyperExtension::initialization_options_from_settings(None),
            None
        );
    }

    #[test]
    fn workspace_configuration_is_none_when_unset() {
        assert_eq!(
            VyperExtension::workspace_configuration_from_settings(None),
            None
        );
    }

    #[test]
    fn prepends_and_deduplicates_env_path() {
        let mut env = vec![(
            "PYTHONPATH".to_string(),
            "/old/site-packages:/workspace/.venv/lib/python3.12/site-packages".to_string(),
        )];

        VyperExtension::prepend_env_path(
            &mut env,
            "PYTHONPATH",
            "/workspace/.venv/lib/python3.12/site-packages",
            ":",
        );

        assert_eq!(
            env,
            vec![(
                "PYTHONPATH".to_string(),
                "/workspace/.venv/lib/python3.12/site-packages:/old/site-packages".to_string(),
            )]
        );
    }

    #[test]
    fn explicit_env_overrides_win_last() {
        let mut env = vec![("PATH".to_string(), "/venv/bin:/usr/bin".to_string())];
        VyperExtension::apply_env_overrides(
            &mut env,
            vec![("PATH".to_string(), "/custom/bin".to_string())],
        );

        assert_eq!(env, vec![("PATH".to_string(), "/custom/bin".to_string())]);
    }
}
