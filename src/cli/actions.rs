use nbd::{
    db::{self, ContactRepo, NoteRepo, Repo},
    models::{self, ContactBuilder, NoteSummary},
};
use sqlx::SqlitePool;
use tabled::Table;

use crate::commander::{
    AddNoteCommand, CreateCommand, DeleteCommand, DeleteNoteCommand, EditCommand, EditNoteCommand,
    GetCommand, ImportCommand, ShowCommand,
};

pub struct Actions {
    data_repo: db::Repo<SqlitePool>,
}

impl Actions {
    pub const fn new(data_repo: Repo<SqlitePool>) -> Self {
        Self { data_repo }
    }

    pub async fn create_contact(&self, command: &CreateCommand) -> Result<(), anyhow::Error> {
        let contact = models::Contact::builder()
            .first_name(command.first_name.as_deref().unwrap_or(""))
            .last_name(command.last_name.as_deref().unwrap_or(""))
            .email(command.email.as_deref().unwrap_or(""))
            .phone_number(command.phone_number.as_deref().unwrap_or(""))
            .birthday(command.birthday.as_deref().unwrap_or(""))
            .build()?;

        // The contact and its notes are saved in a single transaction;
        // a bad note body or a mid-save failure writes nothing at all.
        let id = self
            .data_repo
            .save_contact_with_notes(contact, command.note.clone())
            .await?;

        println!("Successfully saved contact {id}");

        if !command.note.is_empty() {
            let number_of_notes = command.note.len();
            println!("Successfully saved {number_of_notes} note(s)");
        }

        Ok(())
    }

    pub async fn add_note(&self, command: &AddNoteCommand) -> Result<(), anyhow::Error> {
        let note = models::Note::new(command.contact_id, &command.note)?;

        let note_id = self.data_repo.save_note(note).await?;

        println!("Successfully saved note {note_id}");

        Ok(())
    }

    pub async fn edit_note(&self, command: &EditNoteCommand) -> Result<(), anyhow::Error> {
        self.data_repo
            .update_note(command.id, &command.note)
            .await?;

        println!("Note updated");

        Ok(())
    }

    pub async fn delete_note(&self, command: &DeleteNoteCommand) -> Result<(), anyhow::Error> {
        let note_id = self.data_repo.delete_note_by_id(command.id).await?;

        println!("Successfully deleted note {note_id}");

        Ok(())
    }

    pub async fn edit_contact(&self, command: &EditCommand) -> Result<(), anyhow::Error> {
        let contact = ContactBuilder::new(
            command.id,
            command.first_name.clone(),
            command.last_name.clone(),
            command.email.clone(),
            command.phone_number.clone(),
            command.display_name.clone(),
            None,
        )?;

        self.data_repo.update_contact(contact).await?;

        println!("Contact updated");

        Ok(())
    }

    pub async fn show_all_contacts(&self, command: &ShowCommand) -> Result<(), anyhow::Error> {
        let contacts = self.data_repo.get_all_contacts().await?;

        if contacts.is_empty() {
            println!("No contacts yet!");
            return Ok(());
        }

        let table = Table::new(contacts);
        println!("{table}");

        if command.show_notes {
            let notes = self.data_repo.get_all_notes().await?;

            if notes.is_empty() {
                println!("No notes yet!");
            } else {
                println!("Notes:");
                let summaries: Vec<NoteSummary> = notes.iter().map(NoteSummary::from).collect();
                let table = Table::new(summaries);
                println!("{table}");
            }
        }

        Ok(())
    }

    pub async fn get_contact(&self, command: &GetCommand) -> Result<(), anyhow::Error> {
        let id = command.id;

        let contact = self.data_repo.get_contact_by_id(id).await?;

        println!("{contact:?}");

        let notes = self.data_repo.get_notes_for_contact(id).await?;

        if notes.is_empty() {
            println!("No notes yet!");
        } else {
            println!("Notes:");
            for note in notes {
                let created = note.created_at.date_naive();
                println!("#{} ({}):", note.id, created);
                println!("{}", note.body);
            }
        }

        Ok(())
    }

    pub async fn delete_contact(&self, command: &DeleteCommand) -> Result<(), anyhow::Error> {
        let id = command.id;

        let contact_id = self.data_repo.delete_contact_by_id(id).await?;

        println!("Successfully deleted contact {contact_id}");

        Ok(())
    }

    pub async fn import_contacts(&self, command: &ImportCommand) -> Result<(), anyhow::Error> {
        let number_of_imports = self
            .data_repo
            .import_contacts_by_csv(&command.filename)
            .await?;

        println!("Successfully imported {number_of_imports} contact");

        Ok(())
    }
}
