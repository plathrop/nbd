use crate::models;
use async_trait::async_trait;
use sqlx::SqlitePool;

use super::connection::Repo;

/// Persistence seam for notes. This trait exists primarily as a mockall
/// seam for tests; it is not a stability-guaranteed API and may gain
/// methods in any release.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NoteRepo {
    async fn save_note(&self, note: models::Note) -> anyhow::Result<i64>;
    async fn update_note(&self, note_id: i64, body: &str) -> anyhow::Result<()>;
    async fn get_note_by_id(&self, note_id: i64) -> anyhow::Result<models::Note>;
    async fn get_notes_for_contact(&self, contact_id: i64) -> anyhow::Result<Vec<models::Note>>;
    async fn get_all_notes(&self) -> anyhow::Result<Vec<models::Note>>;
    async fn delete_note_by_id(&self, note_id: i64) -> anyhow::Result<i64>;
}

#[async_trait]
impl NoteRepo for Repo<SqlitePool> {
    async fn save_note(&self, note: models::Note) -> anyhow::Result<i64> {
        let contact_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts WHERE id = ?")
            .bind(note.contact_id)
            .fetch_one(&*self.database)
            .await?;

        if contact_count == 0 {
            anyhow::bail!("That Contact ID does not exist");
        }

        let query = "INSERT INTO notes
        (contact_id, body, created_at, updated_at)
        VALUES (?, ?, ?, ?)";

        let result = sqlx::query(query)
            .bind(note.contact_id)
            .bind(&note.body)
            .bind(note.created_at)
            .bind(note.updated_at)
            .execute(&*self.database)
            .await?;

        let note_id = result.last_insert_rowid();

        Ok(note_id)
    }

    async fn update_note(&self, note_id: i64, body: &str) -> anyhow::Result<()> {
        use chrono::Utc;

        let body = body.trim();

        models::Note::validate_body(body)?;

        let now = Utc::now();

        let query_update_note_by_id = "UPDATE notes SET body = ?, updated_at = ? WHERE id = ?";

        let result = sqlx::query(query_update_note_by_id)
            .bind(body)
            .bind(now)
            .bind(note_id)
            .execute(&*self.database)
            .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("That Note ID does not exist");
        }

        Ok(())
    }

    async fn get_note_by_id(&self, note_id: i64) -> anyhow::Result<models::Note> {
        let query_get_by_id = "SELECT * FROM notes WHERE id = ?";

        match sqlx::query_as::<_, models::Note>(query_get_by_id)
            .bind(note_id)
            .fetch_one(&*self.database)
            .await
        {
            Ok(note) => Ok(note),
            Err(sqlx::Error::RowNotFound) => {
                anyhow::bail!("That Note ID does not exist")
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn get_notes_for_contact(&self, contact_id: i64) -> anyhow::Result<Vec<models::Note>> {
        let query_notes_for_contact = "SELECT * FROM notes WHERE contact_id = ? ORDER BY id";

        let notes: Vec<models::Note> = sqlx::query_as::<_, models::Note>(query_notes_for_contact)
            .bind(contact_id)
            .fetch_all(&*self.database)
            .await?;

        Ok(notes)
    }

    async fn get_all_notes(&self) -> anyhow::Result<Vec<models::Note>> {
        let query_get_all_notes = "SELECT * FROM notes ORDER BY id";

        let notes: Vec<models::Note> = sqlx::query_as::<_, models::Note>(query_get_all_notes)
            .fetch_all(&*self.database)
            .await?;

        Ok(notes)
    }

    async fn delete_note_by_id(&self, note_id: i64) -> anyhow::Result<i64> {
        let query_delete_by_id = "DELETE FROM notes WHERE id = ?";

        let result = sqlx::query(query_delete_by_id)
            .bind(note_id)
            .execute(&*self.database)
            .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("That Note ID does not exist");
        }

        Ok(note_id)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::panic_in_result_fn)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::db::ContactRepo;
    use crate::models::OptionalContact;
    use crate::test_helpers::setup_in_memory_db;

    async fn create_contact(data_repo: &Repo<SqlitePool>) -> i64 {
        let contact = OptionalContact {
            first_name: Some("Lewis".to_string()),
            ..OptionalContact::default()
        };

        data_repo.save_optional_contact(contact).await.unwrap()
    }

    #[tokio::test]
    async fn should_save_and_retrieve_a_note() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        let note = models::Note::new(contact_id, "Loves tea parties").expect("Valid note");

        let note_id = data_repo.save_note(note).await?;

        let notes = data_repo.get_notes_for_contact(contact_id).await?;

        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, note_id);
        assert_eq!(notes[0].contact_id, contact_id);
        assert_eq!(notes[0].body, "Loves tea parties");

        Ok(())
    }

    #[tokio::test]
    async fn should_save_multiple_notes_for_one_contact() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        let first = models::Note::new(contact_id, "Met at PyCon").expect("Valid note");
        let second = models::Note::new(contact_id, "Follow up about Rust").expect("Valid note");

        let first_id = data_repo.save_note(first).await?;
        let second_id = data_repo.save_note(second).await?;

        let notes = data_repo.get_notes_for_contact(contact_id).await?;

        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].id, first_id);
        assert_eq!(notes[1].id, second_id);
        assert_eq!(notes[0].body, "Met at PyCon");
        assert_eq!(notes[1].body, "Follow up about Rust");

        Ok(())
    }

    #[tokio::test]
    async fn should_get_all_notes_across_contacts() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let first_contact_id = create_contact(&data_repo).await;

        let second_contact = OptionalContact {
            first_name: Some("Alice".to_string()),
            ..OptionalContact::default()
        };
        let second_contact_id = data_repo.save_optional_contact(second_contact).await?;

        data_repo
            .save_note(models::Note::new(first_contact_id, "First note").expect("Valid note"))
            .await?;
        data_repo
            .save_note(models::Note::new(second_contact_id, "Second note").expect("Valid note"))
            .await?;

        let notes = data_repo.get_all_notes().await?;

        assert_eq!(notes.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn should_return_error_when_saving_note_for_nonexistent_contact() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let note = models::Note::new(999, "Note for nobody").expect("Valid note");

        let result = data_repo.save_note(note).await;

        let err = result.expect_err("Expected save to fail");
        assert!(err.to_string().contains("That Contact ID does not exist"));

        Ok(())
    }

    #[tokio::test]
    async fn should_update_note_body() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        let note = models::Note::new(contact_id, "Original body").expect("Valid note");
        let note_id = data_repo.save_note(note).await?;

        data_repo.update_note(note_id, "Updated body").await?;

        let notes = data_repo.get_notes_for_contact(contact_id).await?;
        let updated = notes.first().expect("Note should exist");

        assert_eq!(updated.body, "Updated body");

        Ok(())
    }

    #[tokio::test]
    async fn should_return_error_when_updating_nonexistent_note() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let result = data_repo.update_note(999, "New body").await;

        let err = result.expect_err("Expected update to fail");
        assert!(err.to_string().contains("That Note ID does not exist"));

        Ok(())
    }

    #[tokio::test]
    async fn should_reject_updating_note_with_invalid_body() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        let note = models::Note::new(contact_id, "Original body").expect("Valid note");
        let note_id = data_repo.save_note(note).await?;

        let result = data_repo.update_note(note_id, "   ").await;

        let err = result.expect_err("Expected update to be rejected");
        assert!(err.to_string().contains("empty"));

        // The original body must be untouched
        let notes = data_repo.get_notes_for_contact(contact_id).await?;
        assert_eq!(
            notes.first().expect("Note should exist").body,
            "Original body"
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_trim_outer_whitespace_when_updating_note() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        let note = models::Note::new(contact_id, "Original body").expect("Valid note");
        let note_id = data_repo.save_note(note).await?;

        data_repo.update_note(note_id, "  Updated body  \n").await?;

        let notes = data_repo.get_notes_for_contact(contact_id).await?;
        assert_eq!(
            notes.first().expect("Note should exist").body,
            "Updated body"
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_get_note_by_id() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        let note = models::Note::new(contact_id, "A memorable note").expect("Valid note");
        let note_id = data_repo.save_note(note).await?;

        let fetched = data_repo.get_note_by_id(note_id).await?;

        assert_eq!(fetched.id, note_id);
        assert_eq!(fetched.body, "A memorable note");

        Ok(())
    }

    #[tokio::test]
    async fn should_return_error_when_getting_nonexistent_note_by_id() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let result = data_repo.get_note_by_id(999).await;

        let err = result.expect_err("Expected get to fail");
        assert!(err.to_string().contains("That Note ID does not exist"));

        Ok(())
    }

    #[tokio::test]
    async fn should_delete_note() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        let note = models::Note::new(contact_id, "Soon deleted").expect("Valid note");
        let note_id = data_repo.save_note(note).await?;

        let deleted_id = data_repo.delete_note_by_id(note_id).await?;

        assert_eq!(deleted_id, note_id);

        let notes = data_repo.get_notes_for_contact(contact_id).await?;

        assert!(notes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn should_return_error_when_deleting_nonexistent_note() {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let result = data_repo.delete_note_by_id(999).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_delete_notes_when_contact_is_deleted() -> anyhow::Result<()> {
        let pool = setup_in_memory_db().await;
        let data_repo = Repo::new(pool);

        let contact_id = create_contact(&data_repo).await;

        data_repo
            .save_note(models::Note::new(contact_id, "Orphan soon").expect("Valid note"))
            .await?;

        data_repo.delete_contact_by_id(contact_id).await?;

        let notes = data_repo.get_all_notes().await?;

        assert!(
            notes.is_empty(),
            "notes should be removed when their contact is deleted"
        );

        Ok(())
    }
}
