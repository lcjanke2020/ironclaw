use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{Shell, generate};
use clap_complete_nushell::Nushell;
use std::io::{self, Write};

/// All shells supported for completion generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    Powershell,
    Zsh,
}

/// Generate shell completion scripts for ironclaw
#[derive(Parser, Debug)]
pub struct Completion {
    /// The shell to generate completions for
    #[arg(value_enum, long)]
    pub shell: CompletionShell,
}

impl Completion {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut cmd = crate::cli::Cli::command();
        let bin_name = cmd.get_name().to_string();

        match self.shell {
            CompletionShell::Zsh => {
                // Generate to buffer so we can patch the compdef call.
                // clap_complete emits bare `compdef _ironclaw ironclaw` which
                // errors if sourced before compinit. Guard it so the script
                // works in all sourcing contexts.
                let mut buf = Vec::new();
                generate(Shell::Zsh, &mut cmd, bin_name.clone(), &mut buf);
                let script = String::from_utf8(buf)?;

                let bare = format!("compdef _{0} {0}", bin_name);
                let guarded =
                    format!("(( $+functions[compdef] )) && compdef _{0} {0}", bin_name);
                let patched = script.replace(&bare, &guarded);

                io::stdout().write_all(patched.as_bytes())?;
            }
            CompletionShell::Nushell => {
                generate(Nushell, &mut cmd, bin_name, &mut io::stdout());
            }
            _ => {
                let shell = match self.shell {
                    CompletionShell::Bash => Shell::Bash,
                    CompletionShell::Elvish => Shell::Elvish,
                    CompletionShell::Fish => Shell::Fish,
                    CompletionShell::Powershell => Shell::PowerShell,
                    _ => unreachable!(),
                };
                generate(shell, &mut cmd, bin_name, &mut io::stdout());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_run_generates_output() {
        let mut cmd = crate::cli::Cli::command();
        let bin_name = cmd.get_name().to_string();
        let mut buf = Vec::new();
        generate(Shell::Zsh, &mut cmd, bin_name, &mut buf);
        assert!(!buf.is_empty(), "generate() should produce output");
    }

    #[test]
    fn test_zsh_compdef_guard_applied() {
        let mut cmd = crate::cli::Cli::command();
        let bin_name = cmd.get_name().to_string();
        let mut buf = Vec::new();
        generate(Shell::Zsh, &mut cmd, bin_name.clone(), &mut buf);
        let raw = String::from_utf8(buf).unwrap();

        // Apply the same patching logic as run()
        let bare = format!("compdef _{0} {0}", bin_name);
        let guarded = format!("(( $+functions[compdef] )) && compdef _{0} {0}", bin_name);
        let patched = raw.replace(&bare, &guarded);

        let bare_compdef = format!("    compdef _{0} {0}\n", bin_name);
        assert!(
            !patched.contains(&bare_compdef),
            "bare compdef should not appear after patching"
        );
        assert!(
            patched.contains("$+functions[compdef]"),
            "patched output should contain compdef guard"
        );
    }

    #[test]
    fn test_nushell_generates_output() {
        let mut cmd = crate::cli::Cli::command();
        let bin_name = cmd.get_name().to_string();
        let mut buf = Vec::new();
        generate(Nushell, &mut cmd, bin_name, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty(), "nushell generate() should produce output");
        assert!(
            output.contains("extern"),
            "nushell completions should contain extern declarations"
        );
    }
}
