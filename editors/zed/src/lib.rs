use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct CpLang;

impl zed::Extension for CpLang {
    fn new() -> Self {
        CpLang
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(id.as_ref(), worktree)
            .ok()
            .and_then(|s| s.binary);
        let command = binary
            .as_ref()
            .and_then(|b| b.path.clone())
            .or_else(|| worktree.which("cpl"))
            .ok_or("`cpl` is not on the PATH; set lsp.cpl.binary.path in Zed's settings")?;
        let args = binary
            .and_then(|b| b.arguments)
            .unwrap_or_else(|| vec!["lsp".into()]);
        Ok(zed::Command {
            command,
            args,
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(CpLang);
