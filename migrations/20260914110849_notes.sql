-- Add notes table for freeform text notes associated with contacts
CREATE TABLE IF NOT EXISTS notes
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contact_id INTEGER NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_notes_contact_id ON notes (contact_id);
