mod contact;
mod note;

pub use contact::Construct as ContactBuilder;
pub use contact::Contact;
pub use contact::Indexed as IndexedContact;
pub use contact::Optional as OptionalContact;
pub use note::{Note, NoteSummary, MAX_NOTE_LENGTH};
