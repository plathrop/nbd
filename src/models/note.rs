use chrono::{DateTime, Utc};
use tabled::Tabled;

/// Maximum allowed length of a note body, in characters.
pub const MAX_NOTE_LENGTH: usize = 10_000;

/// Maximum length of the first-line summary used when rendering tables.
const SUMMARY_MAX_CHARS: usize = 60;

#[derive(Debug, PartialEq, Eq, Clone, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub contact_id: i64,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One-row table rendering of a note: IDs plus a one-line summary of the body.
#[derive(Debug, Tabled)]
pub struct NoteSummary {
    pub id: i64,
    pub contact_id: i64,
    pub note: String,
}

impl From<&Note> for NoteSummary {
    fn from(note: &Note) -> Self {
        Self {
            id: note.id,
            contact_id: note.contact_id,
            note: note.summary(),
        }
    }
}

impl Note {
    /// Validates a note body: it must not be empty (after trimming) and must
    /// not exceed [`MAX_NOTE_LENGTH`] characters.
    ///
    /// # Errors
    ///
    /// This errors if the body is empty or too long.
    pub fn validate_body(body: &str) -> anyhow::Result<()> {
        if body.trim().is_empty() {
            anyhow::bail!("Note cannot be empty");
        }

        if body.chars().count() > MAX_NOTE_LENGTH {
            anyhow::bail!("Note exceeds the maximum length of {MAX_NOTE_LENGTH} characters");
        }

        Ok(())
    }

    /// Builds a new, unsaved note for the given contact.
    ///
    /// The `id` is a placeholder; the repository assigns the real ID on
    /// save. The timestamps are set here and persisted as-is.
    ///
    /// # Errors
    ///
    /// This errors if the body fails [`Note::validate_body`].
    pub fn new(contact_id: i64, body: &str) -> anyhow::Result<Self> {
        Self::validate_body(body)?;

        let now = Utc::now();

        Ok(Self {
            id: 0,
            contact_id,
            body: body.to_owned(),
            created_at: now,
            updated_at: now,
        })
    }

    /// One-line summary of the body: the first line, truncated for display.
    #[must_use]
    pub fn summary(&self) -> String {
        let first_line = self.body.lines().next().unwrap_or_default();

        if first_line.chars().count() > SUMMARY_MAX_CHARS {
            let truncated: String = first_line.chars().take(SUMMARY_MAX_CHARS).collect();
            format!("{truncated}...")
        } else {
            first_line.to_owned()
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {

    use super::{Note, MAX_NOTE_LENGTH};

    #[test]
    fn should_build_a_note() {
        let note = Note::new(7, "Met at PyCon, follow up about Rust").expect("Valid note");

        assert_eq!(note.contact_id, 7);
        assert_eq!(note.body, "Met at PyCon, follow up about Rust");
    }

    #[test]
    fn should_reject_an_empty_note() {
        let result = Note::new(1, "");

        let err = result.expect_err("Expected empty note to be rejected");
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn should_reject_a_whitespace_only_note() {
        let result = Note::new(1, "   \n\t  ");

        assert!(result.is_err());
    }

    #[test]
    fn should_reject_a_note_over_the_length_limit() {
        let body = "x".repeat(MAX_NOTE_LENGTH + 1);

        let result = Note::new(1, &body);

        let err = result.expect_err("Expected over-length note to be rejected");
        assert!(err.to_string().contains("maximum length"));
    }

    #[test]
    fn should_accept_a_note_exactly_at_the_length_limit() {
        let body = "x".repeat(MAX_NOTE_LENGTH);

        let result = Note::new(1, &body);

        assert!(result.is_ok());
    }

    #[test]
    fn should_summarize_note_with_first_line_only() {
        let note = Note::new(1, "First line\nsecond line\nthird line").expect("Valid note");

        assert_eq!(note.summary(), "First line");
    }

    #[test]
    fn should_truncate_long_summaries() {
        let body = "y".repeat(100);
        let note = Note::new(1, &body).expect("Valid note");

        let summary = note.summary();

        assert_eq!(summary.chars().count(), 63);
        assert!(summary.ends_with("..."));
    }
}
