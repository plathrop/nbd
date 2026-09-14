use std::{env, fs, io::Write, path::Path, process::Command};
use tempfile::Builder;

/// Opens the user's editor (`$VISUAL`, then `$EDITOR`) on a temp file
/// preloaded with `initial`, and returns the edited text (trimmed).
///
/// The git-style invocation `sh -c '$EDITOR "$@"' $EDITOR <file>` is used so
/// an `EDITOR` with arguments (e.g. `code --wait`) works.
///
/// # Errors
///
/// This errors if neither `$VISUAL` nor `$EDITOR` is set, if the editor
/// cannot be launched, or if the editor exits with a non-zero status.
pub fn edit_text(extension: &str, initial: &str) -> anyhow::Result<String> {
    let editor = current_editor()?;

    let mut file = Builder::new()
        .prefix("nbd-note")
        .suffix(extension)
        .rand_bytes(6)
        .tempfile()?;

    file.write_all(initial.as_bytes())?;

    let path = file.into_temp_path();

    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$@\""))
        .arg(&editor)
        .arg(path.as_os_str())
        .status()?;

    if !status.success() {
        anyhow::bail!("Editor exited with status {status}");
    }

    let edited = read_text(&path)?;

    Ok(edited.trim().to_owned())
}

fn read_text(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("Failed to read back the edited note: {error}"))
}

fn current_editor() -> anyhow::Result<String> {
    for variable in ["VISUAL", "EDITOR"] {
        if let Ok(editor) = env::var(variable) {
            if !editor.trim().is_empty() {
                return Ok(editor);
            }
        }
    }

    anyhow::bail!(
        "No note text provided and neither $VISUAL nor $EDITOR is set. \
         Set $EDITOR to your editor of choice, or pass the note text on \
         the command line."
    )
}
