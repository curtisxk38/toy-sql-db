
# Toy SQL DB

This project is created as a learning exercise to learn more about the internals of SQL databases (ex Postgres)

I am roughly modeling it after the BusTub project from the CMU 15-445/645 Database Systems course.

## Running

Run the project with `cargo run`. It will drop you into a shell where you can run SQL commands

## Tests

Run tests with `cargo test-db`

This runs the tests with a single thread (instead of the default `cargo test` command running in parallel) because currently the tests default to writing to a single file which fails in parallel.

## Features

Since this is an early WIP the features aren't fully documented, look in the `parse/parser.rs` for what SQL commands are supported.

basics of CREATE TABLE, INSERT INTO ... are working

Features I would like to implement

- [ ] more of SQL standards
- [ ] Hash Index
- [ ] B+Tree Index
- [ ] Multithreading
- [ ] transactions and concurrency control (MVCC ?)
- [ ] client/server architecture
- [ ] WAL/crash recovery?
