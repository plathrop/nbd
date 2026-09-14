use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new contact book
    Init,

    /// Create a contact
    Create(CreateCommand),

    /// Edit a contact by ID
    Edit(EditCommand),

    /// Get all contacts
    Show(ShowCommand),

    /// Get a contact
    Get(GetCommand),

    /// Delete a contact
    Delete(DeleteCommand),

    /// Import contact via CSV
    Import(ImportCommand),

    /// Add a note to a contact
    AddNote(AddNoteCommand),

    /// Edit a note by ID
    EditNote(EditNoteCommand),

    /// Delete a note by ID
    DeleteNote(DeleteNoteCommand),
}

#[derive(Args)]
pub struct CreateCommand {
    #[arg(short, long, value_name = "First Name")]
    pub first_name: Option<String>,

    #[arg(short, long, value_name = "Last Name")]
    pub last_name: Option<String>,

    #[arg(short, long, value_name = "Display Name")]
    pub display_name: Option<String>,

    #[arg(short, long)]
    pub email: Option<String>,

    #[arg(short, long, value_name = "Phone")]
    pub phone_number: Option<String>,

    #[arg(short, long, value_name = "Birthday")]
    pub birthday: Option<String>,

    /// Note(s) to attach to the new contact; may be given multiple times
    #[arg(short, long, value_name = "Note")]
    pub note: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ShowCommand {
    /// Also display contact notes
    #[arg(short, long)]
    pub show_notes: bool,
}

#[derive(Args, Debug)]
pub struct EditCommand {
    /// ID of contact to edit
    pub id: i64,

    #[arg(short, long, value_name = "First Name")]
    pub first_name: Option<String>,

    #[arg(short, long, value_name = "Last Name")]
    pub last_name: Option<String>,

    #[arg(short, long, value_name = "Display Name")]
    pub display_name: Option<String>,

    #[arg(short, long)]
    pub email: Option<String>,

    #[arg(short, long, value_name = "Phone")]
    pub phone_number: Option<String>,
}

#[derive(Args, Debug)]
pub struct GetCommand {
    /// ID of contact to get
    pub id: i64,
}

#[derive(Args, Debug)]
pub struct DeleteCommand {
    /// ID of contact to delete
    pub id: i64,
}

#[derive(Args, Debug)]
pub struct ImportCommand {
    /// name of CSV file
    pub filename: String,
}

#[derive(Args, Debug)]
pub struct AddNoteCommand {
    /// ID of the contact to add the note to
    pub contact_id: i64,

    /// Text of the note
    pub note: String,
}

#[derive(Args, Debug)]
pub struct EditNoteCommand {
    /// ID of the note to edit
    pub id: i64,

    /// New text of the note
    pub note: String,
}

#[derive(Args, Debug)]
pub struct DeleteNoteCommand {
    /// ID of the note to delete
    pub id: i64,
}
