# No Big Deal - A Connection Cultivator

[![Commit Phase](https://github.com/jasonribble/nbd/actions/workflows/ci.yml/badge.svg)](https://github.com/jasonribble/nbd/actions/workflows/ci.yml)

## Description

This is a personal contact management to help people create and maintain thriving relationship in their life.

## Motivation

The world needs a privacy-first, offline-first, personal contact manager.

## Development

This Rust project requires the following:

- [rustup](https://rustup.rs)
- [sqlx-cli](https://github.com/transact-rs/sqlx)

You can also use the nix flake. Install nix through the [nix-installer](https://github.com/DeterminateSystems/nix-installer). Once installed, run:

`$ nix develop`

## Setup

1. Copy the `.env.example` .env

```
cp .env.example .env
```

Optionally update environment variables. I recommend keeping `NBD_CONFIG_DIR` set to the default in `.env.example` so you don't collide with your production use of `nbd`.

1. Create the database.

```
sqlx db create
```

1. Run sqlx migrations

```
sqlx migrate run
```

## Usage

Create a contact

```
Usage: nbd-cli <COMMAND>

Commands:
  create       Create a contact
  edit         Edit a contact by ID
  show         Get all contacts
  get          Get a contact
  delete       Delete a contact
  import       Import contact via CSV
  add-note     Add a note to a contact
  edit-note    Edit a note by ID
  delete-note  Delete a note by ID
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

For example

```bash
cargo run create --first-name test --last-name last --email test@ttest.com --phone-number 123-231-1122 --birthday 1970-01-01
```

Then, you can see the contact using the `show` command

```bash
cargo run show
```

Edit a contact

```
Arguments:
  <ID>  ID of contact to edit

Options:
  -f, --first-name <First Name>
  -l, --last-name <Last Name>
  -d, --display-name <Display Name>
  -e, --email <EMAIL>
  -p, --phone-number <Phone>
  -h, --help                         Print help
```

For example

```bash
cargo run edit 1 -f Jason
```

## Notes

Notes are freeform text attached to a contact - relationship context,
conversation history, or anything else worth remembering.

A note can be added while creating a contact (repeat `--note` for several):

```bash
cargo run create --first-name Ada --last-name Lovelace --note "Met at PyCon"
```

Or later, with `add-note`:

```bash
cargo run add-note 1 "Follow up about Rust"
```

Notes are shown in full with `get`:

```bash
cargo run get 1
```

And summarized in a table with `show --show-notes` (only the first line of
each note is rendered):

```bash
cargo run show --show-notes
```

Edit or delete a note by its note ID (shown by `get` and `show --show-notes`):

```bash
cargo run edit-note 2 "Updated text"
cargo run delete-note 3
```

Notes are limited to 10,000 characters and cannot be empty. Deleting a
contact deletes its notes as well.

## Cleanup

To destroy the database, delete `contacts.db`
