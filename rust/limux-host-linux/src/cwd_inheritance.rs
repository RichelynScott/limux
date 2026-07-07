//! Pure cwd resolution for newly created panes and splits (PRD-H US-2).
//!
//! New terminal panes should start where the user/agent currently is, not at
//! the workspace root (the cmux #7155 class of friction). The precedence is:
//!
//! 1. explicit cwd (a caller-supplied `--cwd`/request value always wins)
//! 2. the source pane's live shell-reported cwd (`term_cwd`, fed by Ghostty's
//!    `GHOSTTY_ACTION_PWD`; requires shell integration to report pwd)
//! 3. the workspace cwd (`folder_path` or tracked workspace cwd)
//! 4. `$HOME`
//!
//! Resolution failure is silent fallback — a blank or missing candidate is
//! skipped, never surfaced as an error to the user.

/// Returns the first usable (non-blank) cwd candidate in precedence order,
/// or `None` when every candidate is absent/blank (callers then keep their
/// existing spawn default).
pub(crate) fn resolve_new_pane_cwd(
    explicit_cwd: Option<&str>,
    source_pane_cwd: Option<&str>,
    workspace_cwd: Option<&str>,
    home_dir: Option<&str>,
) -> Option<String> {
    [explicit_cwd, source_pane_cwd, workspace_cwd, home_dir]
        .into_iter()
        .flatten()
        .find(|candidate| !candidate.trim().is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::resolve_new_pane_cwd;

    #[test]
    fn explicit_cwd_wins_over_all_other_sources() {
        assert_eq!(
            resolve_new_pane_cwd(
                Some("/explicit"),
                Some("/pane"),
                Some("/workspace"),
                Some("/home/user"),
            ),
            Some("/explicit".to_string())
        );
    }

    #[test]
    fn source_pane_cwd_wins_over_workspace_and_home() {
        assert_eq!(
            resolve_new_pane_cwd(
                None,
                Some("/pane/deep/dir"),
                Some("/workspace"),
                Some("/home/user")
            ),
            Some("/pane/deep/dir".to_string())
        );
    }

    #[test]
    fn falls_back_to_workspace_cwd_when_pane_cwd_missing() {
        assert_eq!(
            resolve_new_pane_cwd(None, None, Some("/workspace"), Some("/home/user")),
            Some("/workspace".to_string())
        );
    }

    #[test]
    fn falls_back_to_home_when_pane_and_workspace_missing() {
        assert_eq!(
            resolve_new_pane_cwd(None, None, None, Some("/home/user")),
            Some("/home/user".to_string())
        );
    }

    #[test]
    fn returns_none_when_every_candidate_is_absent() {
        assert_eq!(resolve_new_pane_cwd(None, None, None, None), None);
    }

    #[test]
    fn blank_candidates_are_skipped_not_errors() {
        assert_eq!(
            resolve_new_pane_cwd(
                Some(""),
                Some("   "),
                Some("/workspace"),
                Some("/home/user")
            ),
            Some("/workspace".to_string())
        );
    }

    #[test]
    fn blank_everything_resolves_to_none() {
        assert_eq!(
            resolve_new_pane_cwd(Some(""), Some(" "), Some("\t"), Some("")),
            None
        );
    }
}
