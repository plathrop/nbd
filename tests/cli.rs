#![cfg_attr(test, allow(clippy::panic_in_result_fn))]
mod tests {
    use anyhow::Result;
    use assert_cmd::{cargo, Command};
    use nbd::{
        db::{ContactRepo, NoteRepo, Repo},
        models::Contact,
    };
    use predicates::prelude::PredicateBooleanExt;
    use sqlx::SqlitePool;

    fn create_command() -> Command {
        cargo::cargo_bin_cmd!("nbd-cli")
    }

    fn get_cli_name() -> String {
        let package_name = env!("CARGO_PKG_NAME");
        let cli_name = format!("{package_name}-cli");
        cli_name
    }

    fn get_database_url() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/contacts.db".to_string())
    }

    async fn create_repo() -> Result<Repo<SqlitePool>> {
        let database_url = get_database_url();
        let pool = SqlitePool::connect(&database_url).await?;
        Ok(Repo::new(pool))
    }

    fn create_lewis_carroll_contact() -> Result<Contact> {
        Contact::builder()
            .first_name("Lewis")
            .last_name("Carroll")
            .email("lewis@wonderland.com")
            .phone_number("777-777-7777")
            .birthday("1832-1-27")
            .build()
    }

    fn get_expected_table_header() -> Vec<&'static str> {
        vec![
            "+----+------------+-----------+---------------+----------------------+--------------+------------+",
            "| id | first_name | last_name | display_name  | email                | phone_number | birthday   |",
            "+----+------------+-----------+---------------+----------------------+--------------+------------+",
            "| 1  | Lewis      | Carroll   | Lewis Carroll | lewis@wonderland.com | 777-777-7777 | 1832-01-27 |",
            "+----+------------+-----------+---------------+----------------------+--------------+------------+",
        ]
    }

    async fn clean_database() -> Result<()> {
        let database_url = get_database_url();
        let pool = SqlitePool::connect(&database_url).await?;

        sqlx::query!("PRAGMA foreign_keys = OFF")
            .execute(&pool)
            .await?;

        sqlx::query!("DELETE FROM contacts").execute(&pool).await?;

        sqlx::query("DELETE FROM notes").execute(&pool).await?;

        sqlx::query!("DELETE FROM SQLITE_SEQUENCE WHERE name = 'contacts'")
            .execute(&pool)
            .await?;

        sqlx::query("DELETE FROM SQLITE_SEQUENCE WHERE name = 'notes'")
            .execute(&pool)
            .await?;

        sqlx::query!("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;

        Ok(())
    }

    #[test]
    fn should_have_name_nbd_cli() {
        let crate_name = get_cli_name();
        assert_eq!(crate_name, "nbd-cli");
    }

    #[test]
    fn should_display_cli_help() {
        let mut cmd = create_command();
        cmd.arg("--help");

        let expected_output = [
            "Usage: nbd-cli <COMMAND>",
            "",
            "Commands:",
            "  init         Initialize a new contact book",
            "  create       Create a contact",
            "  edit         Edit a contact by ID",
            "  show         Get all contacts",
            "  get          Get a contact",
            "  delete       Delete a contact",
            "  import       Import contact via CSV",
            "  add-note     Add a note to a contact",
            "  edit-note    Edit a note by ID",
            "  delete-note  Delete a note by ID",
            "  help         Print this message or the help of the given subcommand(s)",
            "",
            "Options:",
            "  -h, --help     Print help",
            "  -V, --version  Print version",
        ];

        cmd.assert()
            .success()
            .stdout(predicates::str::contains(expected_output.join("\n")));
    }

    #[tokio::test]
    async fn should_be_able_to_create_full_contact() {
        clean_database().await.expect("Failed to clean database");

        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("First")
            .arg("--last-name")
            .arg("Last")
            .arg("--email")
            .arg("test@test.com")
            .arg("--phone-number")
            .arg("123-321-1233")
            .arg("--birthday")
            .arg("1970-01-01");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully saved contact"));
    }

    #[tokio::test]
    async fn should_delete_a_contact_when_one_is_present() -> Result<()> {
        clean_database().await.expect("Failed to clean database");

        let data_repo = create_repo().await?;
        let example_contact = create_lewis_carroll_contact()?;

        data_repo.save_contact(example_contact).await?;

        let mut cmd = create_command();
        let contact_id = "1";
        cmd.arg("delete").arg(contact_id);

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully deleted contact"));

        clean_database().await?;

        Ok(())
    }

    #[tokio::test]
    async fn should_show_error_when_deleting_nonexistent_contact() -> Result<()> {
        clean_database().await.expect("Failed to clean database");

        let mut cmd = create_command();
        cmd.arg("delete").arg("999");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("That Contact ID does not exist"));

        Ok(())
    }

    #[test]
    fn should_error_when_providing_invalid_email() {
        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("First")
            .arg("--last-name")
            .arg("Last")
            .arg("--email")
            .arg("test@.com")
            .arg("--phone-number")
            .arg("123-321-1233");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("test@.com is invalid"));
    }

    #[test]
    fn should_error_when_providing_invalid_phone_number() {
        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("First")
            .arg("--last-name")
            .arg("Last")
            .arg("--email")
            .arg("test@com.com")
            .arg("--phone-number")
            .arg("123-321-123");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("123-321-123 is invalid"));
    }

    #[test]
    fn should_error_when_invalid_args() {
        let mut cmd = create_command();
        cmd.arg("First").arg("Last").arg("32321123");

        let stderr = format!("Usage: {} <COMMAND>", get_cli_name());

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains(stderr));
    }

    #[tokio::test]
    async fn should_import_one_contact_when_importing_alice_csv() -> Result<()> {
        clean_database().await.expect("Failed to clean database");

        let mut cmd = create_command();
        cmd.arg("import").arg("tests/fixtures/alice.csv");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully imported 1 contact"));

        let data_repo = create_repo().await?;

        let contacts = data_repo.get_all_contacts().await?;

        assert_eq!(contacts.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn should_import_example_csv_with_three_rows() -> Result<()> {
        clean_database().await?;
        let mut cmd = create_command();
        cmd.arg("import").arg("tests/fixtures/example.csv");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully imported"));

        Ok(())
    }

    #[test]
    fn should_fail_when_given_incorrect_file() {
        let mut cmd = create_command();
        cmd.arg("import").arg("tests/fixtures/example.txt");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("File must have .csv extension"));
    }

    #[test]
    fn should_fail_when_given_blank_csv() {
        let mut cmd = create_command();
        cmd.arg("import").arg("tests/fixtures/blank.csv");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("CSV file is empty"));
    }

    #[tokio::test]
    async fn should_say_no_contacts_when_contacts_are_empty() -> Result<()> {
        clean_database().await?;

        let mut cmd = create_command();
        cmd.arg("show");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("No contacts yet!"));
        Ok(())
    }

    #[tokio::test]
    async fn should_show_one_contact_when_one_contact_available() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        let example_contact = create_lewis_carroll_contact()?;

        data_repo.save_contact(example_contact).await?;

        let mut cmd = create_command();

        cmd.arg("show");

        let expected = get_expected_table_header();

        cmd.assert()
            .success()
            .stdout(predicates::str::contains(expected.join("\n")));
        Ok(())
    }

    #[tokio::test]
    async fn should_show_two_contact_when_two_contact_available() -> Result<()> {
        clean_database().await.expect("Failed to clean database");

        let data_repo = create_repo().await?;
        let example_contact = create_lewis_carroll_contact()?;

        data_repo.save_contact(example_contact.clone()).await?;
        data_repo.save_contact(example_contact).await?;

        let mut cmd = create_command();

        cmd.arg("show");

        let expected = get_expected_table_header();

        cmd.assert()
            .success()
            .stdout(predicates::str::contains(expected.join("\n")));
        Ok(())
    }

    #[tokio::test]
    async fn should_accept_a_firstname_and_birthday() {
        clean_database().await.expect("Failed to clean database");

        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("Molly")
            .arg("--birthday")
            .arg("1970-01-01");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully saved contact"));
    }

    #[tokio::test]
    async fn should_set_birthday_when_provided() -> Result<()> {
        clean_database().await.expect("Failed to clean database");

        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("nbd")
            .arg("--birthday")
            .arg("2024-06-06");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully saved contact"));

        let data_repo = create_repo().await?;

        let contact = data_repo.get_contact_by_id(1).await?.contact;

        let birthday =
            chrono::NaiveDate::from_ymd_opt(2024, 6, 6).expect("Should be June, 6, 2024");

        assert_eq!(contact.birthday, birthday);

        Ok(())
    }

    #[tokio::test]
    async fn should_allow_only_first_name_when_creating() -> Result<()> {
        clean_database().await.expect("Failed to clean database");

        let mut cmd = create_command();
        cmd.arg("create").arg("--first-name").arg("Test");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully saved contact"));

        Ok(())
    }

    #[tokio::test]
    async fn should_say_already_initialized_when_db_exists() -> Result<()> {
        let mut first_cmd = create_command();
        first_cmd.arg("init");
        first_cmd.assert().success();

        let mut second_cmd = create_command();
        second_cmd.arg("init");

        second_cmd
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "A contact book has already been initialized",
            ));

        Ok(())
    }

    #[tokio::test]
    async fn should_init_create_database_on_fresh_system() -> Result<()> {
        let temp = tempfile::TempDir::new()?;
        let config_dir = temp.path().to_path_buf();

        let mut cmd = create_command();
        cmd.env("NBD_CONFIG_DIR", &config_dir);

        cmd.arg("init");

        cmd.assert().success();
        let db_path = config_dir.join("contacts.db");
        assert!(db_path.exists(), "expected database file at {db_path:?}");

        Ok(())
    }

    #[tokio::test]
    async fn should_create_contact_with_notes() -> Result<()> {
        clean_database().await?;

        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("Alice")
            .arg("--last-name")
            .arg("Lovelace")
            .arg("--note")
            .arg("Met at PyCon")
            .arg("--note")
            .arg("Follow up about Rust");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully saved contact"))
            .stdout(predicates::str::contains("Successfully saved 2 note(s)"));

        let data_repo = create_repo().await?;

        let notes = data_repo.get_notes_for_contact(1).await?;

        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].body, "Met at PyCon");
        assert_eq!(notes[1].body, "Follow up about Rust");

        Ok(())
    }

    #[tokio::test]
    async fn should_reject_note_over_maximum_length_on_create() -> Result<()> {
        clean_database().await?;

        let long_note = "x".repeat(nbd::models::MAX_NOTE_LENGTH + 1);

        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("Alice")
            .arg("--note")
            .arg(&long_note);

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("maximum length"));

        // The contact must not have been created with a bad note
        let data_repo = create_repo().await?;
        let contacts = data_repo.get_all_contacts().await?;
        assert!(contacts.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn should_reject_empty_note_on_create() -> Result<()> {
        clean_database().await?;

        let mut cmd = create_command();
        cmd.arg("create")
            .arg("--first-name")
            .arg("Alice")
            .arg("--note")
            .arg("   ");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("Note cannot be empty"));

        Ok(())
    }

    #[tokio::test]
    async fn should_add_note_to_existing_contact() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note").arg("1").arg("Loves wordplay");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully saved note 1"));

        let notes = data_repo.get_notes_for_contact(1).await?;

        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "Loves wordplay");

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_when_adding_note_to_nonexistent_contact() -> Result<()> {
        clean_database().await?;

        let mut cmd = create_command();
        cmd.arg("add-note").arg("999").arg("Hello?");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("That Contact ID does not exist"));

        Ok(())
    }

    #[tokio::test]
    async fn should_edit_note() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note").arg("1").arg("Original body");
        cmd.assert().success();

        let mut cmd = create_command();
        cmd.arg("edit-note").arg("1").arg("Edited body");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Note updated"));

        let notes = data_repo.get_notes_for_contact(1).await?;
        assert_eq!(notes[0].body, "Edited body");

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_when_editing_nonexistent_note() -> Result<()> {
        clean_database().await?;

        let mut cmd = create_command();
        cmd.arg("edit-note").arg("999").arg("New body");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("That Note ID does not exist"));

        Ok(())
    }

    #[tokio::test]
    async fn should_delete_note() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note").arg("1").arg("Temporary note");
        cmd.assert().success();

        let mut cmd = create_command();
        cmd.arg("delete-note").arg("1");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully deleted note 1"));

        let notes = data_repo.get_notes_for_contact(1).await?;
        assert!(notes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_when_deleting_nonexistent_note() -> Result<()> {
        clean_database().await?;

        let mut cmd = create_command();
        cmd.arg("delete-note").arg("999");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("That Note ID does not exist"));

        Ok(())
    }

    #[tokio::test]
    async fn should_show_notes_when_getting_a_contact() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note")
            .arg("1")
            .arg("Wrote Alice in Wonderland");
        cmd.assert().success();

        let mut cmd = create_command();
        cmd.arg("get").arg("1");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Notes:"))
            .stdout(predicates::str::contains("Wrote Alice in Wonderland"));

        Ok(())
    }

    #[tokio::test]
    async fn should_say_no_notes_when_getting_contact_without_notes() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("get").arg("1");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("No notes yet!"));

        Ok(())
    }

    #[tokio::test]
    async fn should_show_notes_with_show_notes_flag() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note")
            .arg("1")
            .arg("First line of note\nsecond line stays hidden in summaries");
        cmd.assert().success();

        let mut cmd = create_command();
        cmd.arg("show").arg("--show-notes");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Notes:"))
            .stdout(predicates::str::contains("First line of note"))
            .stdout(predicates::str::contains("| note"))
            // only the first line is rendered in the summary table
            .stdout(predicates::str::contains("second line stays hidden").not());

        Ok(())
    }

    #[tokio::test]
    async fn should_not_show_notes_by_default_in_show() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note").arg("1").arg("Secret note body");
        cmd.assert().success();

        let mut cmd = create_command();
        cmd.arg("show");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Secret note body").not());

        Ok(())
    }

    #[tokio::test]
    async fn should_delete_notes_when_contact_is_deleted() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        let contact_id = data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note")
            .arg(contact_id.to_string())
            .arg("Note that should not outlive the contact");
        cmd.assert().success();

        let mut cmd = create_command();
        cmd.arg("delete").arg(contact_id.to_string());
        cmd.assert().success();

        let notes = data_repo.get_all_notes().await?;
        assert!(notes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn should_compose_note_in_editor_when_text_is_omitted() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.env("VISUAL", "cp tests/fixtures/editor_note.txt")
            .arg("add-note")
            .arg("1");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Successfully saved note 1"));

        let notes = data_repo.get_notes_for_contact(1).await?;

        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "Composed in an editor\nwith a second line");

        Ok(())
    }

    #[tokio::test]
    async fn should_edit_note_in_editor_prefilled_with_current_text() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.arg("add-note").arg("1").arg("Original body");
        cmd.assert().success();

        let captured_path = std::path::PathBuf::from("tests/captured_editor_body.txt");

        let mut cmd = create_command();
        cmd.env("VISUAL", "sh tests/fixtures/capture_editor.sh")
            .arg("edit-note")
            .arg("1");

        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Note updated"));

        let captured = std::fs::read_to_string(&captured_path)?;
        assert_eq!(captured, "Original body");

        std::fs::remove_file(&captured_path)?;

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_when_editor_exits_nonzero() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.env("VISUAL", "false").arg("add-note").arg("1");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("Editor exited with status"));

        let notes = data_repo.get_notes_for_contact(1).await?;
        assert!(
            notes.is_empty(),
            "nothing should be saved when the editor aborts"
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_when_editor_leaves_note_empty() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        // `true` exits 0 but leaves the editor buffer untouched (empty)
        let mut cmd = create_command();
        cmd.env("VISUAL", "true").arg("add-note").arg("1");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("Note cannot be empty"));

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_when_no_editor_is_set() -> Result<()> {
        clean_database().await?;

        let data_repo = create_repo().await?;
        data_repo
            .save_contact(create_lewis_carroll_contact()?)
            .await?;

        let mut cmd = create_command();
        cmd.env_remove("VISUAL")
            .env_remove("EDITOR")
            .arg("add-note")
            .arg("1");

        cmd.assert().failure().stderr(predicates::str::contains(
            "neither $VISUAL nor $EDITOR is set",
        ));

        Ok(())
    }
}
