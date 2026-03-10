use zed_extension_api::{
    Command, Extension, LanguageServerId, LanguageServerInstallationStatus, Result, Worktree,
    register_extension, serde_json, set_language_server_installation_status, settings::LspSettings,
};

struct VyperExtension;

struct LaunchSettings {
    args: Vec<String>,
    env: Vec<(String, String)>,
    explicit_path: Option<String>,
}

impl VyperExtension {
    const LANGUAGE_SERVER_ID: &'static str = "vyper-lsp";

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

    fn mark_status(
        language_server_id: &LanguageServerId,
        status: LanguageServerInstallationStatus,
    ) {
        set_language_server_installation_status(language_server_id, &status);
    }

    fn missing_binary_message() -> String {
        "Unable to find `vyper-lsp`. Install it with `uv tool install vyper-lsp`, ensure it is available on your PATH, or configure `lsp.vyper-lsp.binary.path`.".to_string()
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
        let launch = Self::launch_settings(worktree);

        if let Some(explicit_path) = launch.explicit_path {
            Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
            return Ok(Command {
                command: explicit_path,
                args: launch.args,
                env: launch.env,
            });
        }

        if let Some(path_binary) = worktree.which(Self::LANGUAGE_SERVER_ID) {
            Self::mark_status(language_server_id, LanguageServerInstallationStatus::None);
            return Ok(Command {
                command: path_binary,
                args: launch.args,
                env: launch.env,
            });
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
