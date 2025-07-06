# Refinement Tracker

This document aggregates open critiques and suggested improvements from the latest code review. Items are grouped by priority to guide future work.

## High Priority
- ~~**Transactional migrations**: wrap generated SQL in `BEGIN`/`COMMIT` blocks so partial failures do not leave the database in an inconsistent state.~~
- ~~**`migrate apply` command**: add a CLI command that applies migrations using `sqlx`, completing the workflow without external tools.~~
- ~~**Improve introspection**:~~
  - ~~Parse index definitions via `pg_index` and `pg_attribute` instead of string parsing.~~
  - ~~Allow configurable type mappings instead of the current hard coded match list.~~
- ~~**Plugin security warnings**: prominently document the risk of running arbitrary executables from the schema and surface warnings in CLI help text.~~
- ~~**Hash-based migration tracking**: store a hash for each applied migration and refuse to run if the file changes.~~

## Medium Priority
- ~~**Complete generated CRUD code**: replace `todo!()` placeholders with real `sqlx` queries for `create`, `find`, `update`, `delete`, and `list` helpers.~~
- ~~**Data migrations**: support generating and executing migrations that transform existing data.~~
- ~~**GraphQL query and mutation generation**: emit resolvers alongside the schema types.~~
- ~~**Lint opaque definitions**: warn when constraints or other fields contain strings the tool cannot interpret (e.g. EXCLUDE constraint definitions).~~
- ~~**Typed create errors**: generate model-specific error enums mapping unique and foreign key constraints to variants.~~

## Low Priority
- ~~**Additional database backends**: abstract SQL generation and introspection to support MySQL, SQLite, etc.~~
- ~~**Structured schema references**: replace string based relation targets with structured tables to reduce typos.~~
- **Web or GUI editor**: provide a higher level interface for editing schemas and managing the project.
- ~~**Macro system**: allow reusable blocks or higher level DSL constructs to reduce boilerplate in large schemas.~~

These points summarize the main areas for refinement identified so far. Progress on each item should be logged in `progress.md` as work continues.
