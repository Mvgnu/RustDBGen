# Progress Log

## Phase 0 - Proof of Concept
- Initialized Rust binary project.
- Defined initial Intermediate Representation (IR) for schema in `src/ir.rs`.
- Added `schema.model.toml` with one `User` model.
- Implemented simple CLI in `main.rs` that parses the schema and prints generated Rust struct and a placeholder create function.

## Phase 1 - CLI and Codegen Expansion
- Added a Clap-based CLI with a `generate` subcommand.
- `generate` reads `schema.model.toml` and writes the generated code to a file or stdout.
- Expanded the generator to emit placeholder CRUD helpers (`create`, `find`, `update`, `delete`, `list`).

## Phase 2 - Model Helper Structs
- Added generation of `New` and `Update` structs for each model excluding fields with defaults.
- Exported `generate_code` from a new library crate so tests can call it.
- Created first integration test verifying the new structs appear in the output.

## Phase 3 - Dynamic Update Queries
- Implemented dynamic SQL generation for `update` using `QueryBuilder`.
- Updated integration test to verify `QueryBuilder` usage.

## Phase 4 - Initial Migration Generation
- Added `migrate generate` CLI command with timestamped file output.
- Implemented simple migration generator that creates `CREATE TABLE` and `DROP TABLE` statements.
- New integration test ensures the migration SQL includes table creation.

## Phase 5 - Basic Schema Diffing
- Added `generate_migration` function that compares the current schema with a previous schema snapshot.
- CLI now saves `migrations/schema.json` and uses it to produce ALTER TABLE statements for added or removed columns and tables.
- New integration test validates that diff generation outputs `ADD COLUMN` and corresponding `DROP COLUMN` statements.

## Phase 6 - Column Type Diffing
- Extended `generate_migration` to detect column type changes and emit `ALTER COLUMN ... TYPE` statements.
- Added integration test covering a type change scenario.

## Phase 7 - Default and Nullability Diffing
- The migration generator now tracks changes to column defaults and nullability.
- New tests verify `SET DEFAULT`, `DROP DEFAULT`, `SET NOT NULL`, and `DROP NOT NULL` statements.

## Phase 8 - Index and Foreign Key Diffing
- Added `IndexDef` and `RelationDef` to the schema IR.
- Initial and diff migration functions now emit `CREATE INDEX`, `DROP INDEX`,
  `ADD CONSTRAINT ... FOREIGN KEY`, and matching drop statements.
- Added integration tests for index and foreign key migrations.

## Phase 9 - Unique Index Diffing
- `IndexDef` now has a `unique` flag.
- Migrations create `UNIQUE INDEX` when needed and detect changes to index fields or uniqueness.
- Updated schema example with a unique index on `User.email`.
- Added integration tests for index diffs to account for the new flag.

## Phase 10 - Unique Constraint Diffing
- Introduced `UniqueConstraintDef` for multi-column unique constraints.
- Initial and diff migrations now emit `ADD CONSTRAINT ... UNIQUE` statements.
- Updated example schema with a compound unique constraint on `Post.title` and `author_id`.
- Added tests covering addition and modification of unique constraints.

## Phase 11 - Check Constraint Diffing
- Added `CheckConstraintDef` to the schema representation.
- Example schema demonstrates new check constraints on `User.email` and `Post.title`.
- Migration generation handles check constraints in initial output and when diffing.
- Added integration tests for adding and modifying check constraints.

## Phase 12 - Exclusion Constraint Diffing
- Added `ExclusionConstraintDef` supporting arbitrary `EXCLUDE` clauses.
- Example schema now includes a sample exclusion constraint on `Post`.
- Migration generation emits `ADD CONSTRAINT ... EXCLUDE` and diffs definition changes.
- Integration tests cover creation, alteration, and removal of exclusion constraints.

## Phase 13 - Database Introspection
- Added an asynchronous `introspect_schema` function that queries `information_schema` tables
  and builds a basic `SchemaIR` from an existing Postgres database.
- The CLI gained a new `introspect` subcommand which writes the discovered schema to stdout
  or a specified file.
- Current introspection captures only tables and columns but lays groundwork for indexes
  and constraints in future phases.

## Phase 14 - Introspect Indexes and Constraints
- `introspect_schema` now queries `pg_indexes`, `information_schema.table_constraints`,
  and `pg_constraint` to capture indexes, unique, check, exclusion, and foreign
  key constraints for each table.
- The resulting `SchemaIR` includes these definitions so introspected schemas can
  round trip through migration generation.
- No tests yet exercise this due to needing a live database, but the CLI can now
  produce a complete schema snapshot.

## Phase 15 - Database-Aware Migration Generation
- `migrate generate` accepts an optional `--url` flag. When provided (or when
  `DATABASE_URL` is set) the command introspects the database and uses that
  schema as the baseline for diffing instead of `migrations/schema.json`.
- This enables generating migrations against an existing database without a
  prior snapshot on disk.
- Updated CLI implementation accordingly. No tests cover the new flag because it
  requires a running PostgreSQL instance.
## Phase 16 - Database-Backed Tests
- Added `pg-embed` as a dev dependency to spin up a temporary Postgres instance during tests.
- New `db_introspection` integration test launches Postgres, applies the initial migration,
  and verifies that `introspect_schema` plus `generate_migration` produce no diff.

## Phase 17 - Migration Check Command
- Added a `migrate check` CLI command that diffs the schema without creating files.
- The command exits with an error when changes are pending, enabling CI enforcement.
- Marked the database-backed test with `#[ignore]` to avoid failures when Postgres binaries cannot be fetched.

## Phase 18 - Schema Linting
- Added a `lint` CLI command that validates model references in the schema.
- New `lint_schema` function checks relation, index, and unique constraint fields.
- Added integration tests covering a valid schema and an invalid relation.

## Phase 19 - Model Options
- Introduced `ModelOptions` allowing per-model flags like `timestamps` and `soft_delete`.
- `apply_model_options` expands these flags into standard fields (`created_at`, `updated_at`, `deleted_at`).
- CLI now applies these options before codegen, migrations, or linting.
- Example schema updated to demonstrate the new syntax.
- Added tests verifying option expansion.

## Phase 20 - Field Tags
- `FieldDef` now includes a `tags` vector allowing arbitrary annotations per field.
- Example schema marks `User.email` as `searchable` and `Post.title` with a `display_name` tag.
- Updated `apply_model_options` and introspection helpers to initialize empty tag lists for generated fields.
- Adjusted all unit tests to construct `FieldDef` with the new field.

## Phase 21 - Enum Support
- Added `EnumDef` and an `enums` map on `SchemaIR`.
- Schema can now declare enums like `PostStatus` and reference them from models.
- Code generation emits Rust enums deriving `sqlx::Type` and serde traits.
- Migration generation handles creating, dropping, and updating enum types.
- Database introspection reads existing enums from `pg_type` and `pg_enum`.
 - Example schema introduces a `PostStatus` enum and a `status` field on `Post`.

## Phase 22 - Type Aliases
- Added `TypeAlias` definitions loaded from `type_map.toml`.
- Introduced `apply_type_aliases` to replace field types based on the alias map.
- `FieldDef.db_type` is now optional and filled by alias expansion or introspection.
- Example schema now uses an `Email` alias for `User.email`.
- CLI reads `type_map.toml` before applying model options.

## Phase 23 - Schema Includes
- Added a `load_schema` helper that resolves an optional `include` array to merge referenced files.
- Split `User` and `Post` definitions into `models/user.toml` and `models/post.toml` included from the root schema.
- CLI and tests now load the schema via `load_schema` so includes are processed.

## Phase 24 - Model Permissions
- Added a `Permissions` struct allowing lists of roles for `read`, `update`, and `delete` actions on each model.
- `ModelDef` gained a `permissions` field defaulting to empty lists.
- Example schema demonstrates usage on `User` with admin and member roles.
- Tests verify that the permissions block is parsed correctly.

## Phase 25 - Route Definitions
- Added a `RouteDef` type and a `routes` map on `SchemaIR`.
- Schema files can define `[routes.*]` blocks specifying path, allowed methods, and whether auth is required.
- Code generation now emits a `Route` struct and constants for each route.
- Example schema defines routes for `User` and `Post` models.
- New test ensures route constants appear in the generated code.

## Phase 26 - TypeScript Generation
- Implemented a `generate_typescript` function that outputs interfaces for each model and a `routes` map.
- Added a `generate-ts` CLI subcommand to write the TypeScript file.
- Basic Rust-to-TypeScript type mapping converts primitives and `Option<T>`.
- Tests verify that TypeScript output includes model interfaces and route definitions.

## Phase 27 - GraphQL Schema Generation
- Implemented a `generate_graphql_schema` function that outputs GraphQL types for
  enums, models, and relations.
- Added a `generate-graphql` CLI subcommand to write the schema.
- Basic Rust-to-GraphQL mapping handles primitives and optional fields.
- New test verifies the Post type includes an `author` relation field.

## Phase 28 - Zod Schema Generation
- Extended TypeScript generation to also emit Zod validation schemas for each model.
- Added optional `zod` field annotation in the schema; `User.email` demonstrates `z.string().email()`.
- Updated tests to check for generated Zod schemas in the TypeScript output.

## Phase 29 - Relation Update Diffing
- `generate_migration` now detects changes to existing foreign key relations.
- When a relation's `on` or `references` target changes, the migration drops and
  recreates the constraint with the new definition while producing a matching
  reverse step.
- Added a new `diff_migration_changes_foreign_key` test covering this behavior.

## Phase 30 - Column Rename Diffing
- `FieldDef` gained an optional `rename_from` attribute used to mark renamed
  columns in the schema.
- Migration generation detects when a field specifies `rename_from` and emits
  `ALTER TABLE ... RENAME COLUMN` statements while also applying type, default,
  or nullability changes for the new column.
- Added a `diff_migration_renames_column` test demonstrating the new behavior.

## Phase 31 - Database Rename Tests
- Added `rename_column_round_trip` integration test spinning up a temporary
  Postgres instance with `pg-embed`.
- The test applies the initial schema, renames `User.email` to
  `contact_email` via the IR, and verifies migration generation produces the
  expected `ALTER TABLE ... RENAME COLUMN` statements.
- After applying the migration the test re-introspects the database to ensure
  no further diffs remain. The test is `#[ignore]` when Postgres binaries are
  unavailable.

## Phase 32 - Check Constraint Round Trip
- Added `check_constraint_round_trip` integration test using `pg-embed`.
- The test modifies the `post_title_length` check constraint in-memory,
  generates a migration against the introspected schema, applies it, and
  verifies no further diffs remain after applying.
- This expands live database coverage beyond column renames.

## Phase 33 - Unique Constraint and Index Round Trip
- Added `unique_constraint_round_trip` and `index_uniqueness_round_trip` integration tests using `pg-embed`.
- These tests modify a unique constraint and the uniqueness of an index, generate migrations, apply them, and verify the schema is stable after introspection.
- The tests are ignored by default so they only run when Postgres binaries are available.


## Phase 34 - Plugin Execution Command
- Introduced a `plugin` CLI subcommand that runs an external executable with the schema IR serialized to JSON on stdin.
- Added `run_plugin` helper in the library to handle process spawning and output capture.
- Example usage allows piping the schema through utilities like `cat` for custom generation.
- Added a new test verifying that running the plugin with `cat` outputs valid JSON.

## Phase 35 - Plugin Definitions
- Schema files can now include `[plugins.<name>]` sections specifying a command to execute.
- `SchemaIR` gained a `plugins` map and new `PluginDef` type.
- The `plugin` CLI subcommand accepts either `--exe` or `--name` to run a plugin by path or by schema name.
- `load_schema` merges plugin definitions from included files.
- Added tests verifying plugin definitions parse correctly and that running a named plugin using `cat` works.

## Phase 36 - Enhanced Linting
- `lint` now detects duplicate route paths and warns when relations lack a reciprocal relation on the referenced model.
- The example schema defines a `User.posts` relation so lint passes.
- New tests cover the reciprocal relation and duplicate route path errors.

## Phase 37 - Plugin Args and Discovery
- `PluginDef` supports an `args` array so schemas can provide default arguments.
- `run_plugin` accepts an argument slice and forwards it to the executable.
- The `plugin` subcommand merges schema arguments with `--arg` values and looks up executables by name using the system `PATH` when not found in the schema.
- An `echo` plugin in the example schema demonstrates default arguments.
- Added tests for argument passing and path discovery via the `which` crate.

## Phase 38 - Check Constraint and Route Linting
- Extended `lint_schema` to validate that check constraint expressions reference at least one known field and are not empty.
- Added validation that each route method is one of GET, POST, PUT, DELETE, or PATCH.
- New tests cover bad check expressions and unsupported HTTP methods.

## Next Steps
- Explore additional lint rules and plugin improvements.

## Phase 39 - Route Permissions
- `RouteDef` now contains a `permissions` block mirroring model permissions.
- Rust code generation emits a `Permissions` struct and includes permissions in route constants.
- TypeScript generation outputs each route with its permissions for frontend use.
- Example route definitions demonstrate read/update/delete role lists.
- Tests ensure route permissions parse correctly and appear in generated outputs.

## Phase 40 - Storage Options
- Introduced a `StorageOptions` struct for file fields specifying backend, optional max size, allowed MIME types, and path.
- `FieldDef` now has an optional `storage` attribute using this struct.
- Example schema includes `User.profile_pic` and `Post.attachment` using S3 storage.
- Added a `File` type alias in `type_map.toml` expanding to `String`/`TEXT`.
- Tests cover parsing of storage options from the schema.

## Phase 41 - Enum Reference Lint
- `lint_schema` now verifies that any field using an enum type references an enum defined in the schema.
- Added defaults for builtin types so only unknown enums trigger errors.
- Updated tests to account for the new `storage` field and added coverage for the enum lint rule.
- `run_plugin` ignores `BrokenPipe` errors when a plugin doesn't read stdin, fixing the echo plugin test.

## Phase 42 - Permission Role Lint
- Added a `Role` enum listing valid roles (`admin`, `member`, `guest`).
- `lint_schema` checks model and route permission blocks against this enum and reports unknown roles.
- Example schema updated with the new enum.
- Added tests covering invalid role references.

## Phase 43 - Plugin Environment Variables
- `PluginDef` now supports an optional `env` table for specifying environment variables.
- `run_plugin` accepts these variables and sets them when spawning the process.
- CLI `plugin` subcommand passes the env map from the schema.
- Example schema defines an `envtest` plugin demonstrating this feature.
- Added a new test `plugin_with_env_vars` verifying plugin environment handling.

## Phase 44 - Plugin Working Directory
- Added optional `cwd` field on `PluginDef` specifying the working directory.
- `run_plugin` now accepts the `cwd` option and sets the process directory.
- CLI `plugin` command includes a `--cwd` flag overriding the schema value.
- Added `pwdtest` plugin example using `cwd` to emit its working directory.
- New test `plugin_with_cwd` confirms the plugin runs in the specified directory.

## Phase 45 - Duplicate Plugin Detection
- `load_schema` now reports an error if multiple included files define a plugin with the same name.
- Updated tests to cover this error case using temporary schema fragments.
- Added `tempfile` as a dev-dependency for creating temp directories during tests.

## Phase 46 - Duplicate Model and Enum Detection
- Extended `load_schema` to check for duplicate models, enums, and routes when processing included files.
- New tests ensure errors are raised when an included file defines the same model or enum twice.

## Phase 47 - Seed Data Generation
- Added `SeedDef` structures and a `seeds` map on `SchemaIR` for declaring initial data rows.
- New CLI `seed` subcommand emits SQL `INSERT` statements from these definitions.
- Example schema provides seed data for `User` and `Post` models.
- Implemented `generate_seed_sql` helper and accompanying test.

## Phase 48 - Seed Linting
- `lint_schema` now validates seed data blocks, ensuring each block references an existing model and that all row fields exist on that model.
- Added tests for unknown seed models and fields to verify lint failures.

## Phase 49 - Seed Application
- Added `apply_seed_data` helper executing generated seed SQL using `sqlx`.
- CLI `seed` command now accepts a `--url` option; when provided, seeds are inserted into the database instead of just printing SQL.
- New ignored integration test `apply_seed_inserts_rows` runs seeds against a temporary Postgres instance.


## Phase 50 - Schema Registry Commands
- Added `push_schema` and `pull_schema` helpers for copying schemas to and from a registry path.
- New CLI `registry` subcommand with `push` and `pull` actions, using `SCHEMA_REGISTRY_PATH` when no path flag is given.
- Created `registry_push_pull_round_trip` integration test verifying schema files round trip via a temporary directory.

## Phase 51 - Observability Instrumentation
- Added `observability_provider` option under `[meta]` in the schema.
- When set to `"tracing"`, code generation decorates CRUD methods with `#[tracing::instrument]`.
- Updated schema to enable tracing and extended tests to check for the attribute.

## Phase 52 - TypeScript Client Generation
- Implemented `generate_ts_client` producing a fetch-based API client for all routes.
- New CLI subcommand `generate-client` writes the client to a file or stdout.
- Example tests ensure the client contains wrapper functions for each route.

## Phase 53 - Error Handling Cleanup
- Replaced remaining `unwrap` calls when loading included schemas with proper
  error reporting for non-UTF8 paths.
- GraphQL generation no longer panics if a relation string is malformed.
- `introspect_schema` now returns `anyhow::Result` for consistency across the
  library.

## Phase 54 - Crate Name Normalization
- Renamed the crate from `RustDBGen` to `rustdbgen` to follow Rust naming conventions.
- Updated all references in source files, tests, and Cargo manifests.
- Regenerated `Cargo.lock` and verified build and tests succeed without warnings.

## Phase 55 - Clippy Cleanup
- Fixed clippy warnings by using `split_once` for exclusion constraint parsing
- Iterated over models with `values_mut` in `apply_model_options`
- Removed unused `serde_json` import

## Phase 56 - Include Cycle Detection
- Added cycle detection to `load_schema` using a visited set to prevent recursive includes.
- New test `include_cycle_errors` ensures cyclic includes produce an error.

## Phase 57 - Canonical Include Paths
- `load_schema` now canonicalizes file paths before processing includes.
- This prevents false cycle errors when the same file is referenced via different relative paths.

## Phase 58 - PathBuf Include Tracking
- `load_schema` now tracks visited files using `PathBuf` for better platform compatibility.
- Added context to path errors when reading or canonicalizing schema files.

## Phase 59 - Deterministic Codegen Imports
- Code generation now emits imports for `chrono`, `uuid`, and `sqlx::FromRow` based on field types
- Enums, models, fields, and routes are processed in sorted order for stable output
- Struct derives include schema default derives plus `sqlx::FromRow`


## Phase 60 - Deterministic Migration Ordering
- `generate_initial_migration` and `generate_migration` now sort enums, models, fields, and constraints before emitting SQL
- Ensures migration files are stable across runs for the same schema

## Phase 61 - Refinement Tracker
- Created `refinement.md` summarizing outstanding critiques and improvement ideas from the recent audit.
- This file categorizes tasks by priority to guide future development.

## Phase 62 - Transactional Migrations
- `generate_initial_migration` and `generate_migration` now wrap emitted SQL in
  `BEGIN` and `COMMIT` blocks.
- Empty diffs still return empty strings so `migrate check` behaves correctly.
- Updated tests to verify the transaction wrappers in generated migrations.


## Phase 63 - Migration Apply Command
- Added `apply_migrations` helper executing pending `.up.sql` files and tracking them in a migrations table
- Introduced `migrate apply` CLI subcommand with a `--url` option
- Updated refinement tracker to mark the item complete

## Phase 64 - Improved Introspection
- `introspect_schema` queries `pg_index` and `pg_attribute` for index details instead of parsing strings
- Database type mapping is now loaded from `[db_types]` in `type_map.toml`
- CLI passes the mapping when introspecting
- Marked the refinement item as complete

## Phase 65 - Plugin Security Warning
- Added a security warning to the plugin subcommand help text and printed a runtime notice before execution
- README now contains a note advising caution when running plugins
- Marked the refinement tracker item complete

## Phase 66 - Complete CRUD Generation
- Replaced `todo!` placeholders in generated code with working SQLx queries for `create`, `find`, `delete`, and `list`.
- Generation handles tables with no explicit insert columns via `DEFAULT VALUES`.
- Added a unit test ensuring CRUD SQL snippets appear in generated output.
- Marked the refinement tracker item for complete CRUD code as finished.

## Phase 67 - Data Migration Scaffolding
- Added `create_data_migration` helper to generate timestamped `.data.sql` files
- `migrate generate-data` CLI subcommand creates a skeleton data migration
- `apply_migrations` now runs `.data.sql` files after schema migrations
- New ignored integration test verifies data migrations execute against Postgres
- Documented the new command in the README

## Phase 68 - GraphQL Query and Mutation Generation
- `generate_graphql_schema` now emits `Query` and `Mutation` types with CRUD operations
- Input objects `Create*Input` and `Update*Input` are generated for each model
- Added tests verifying the new GraphQL definitions
- Marked the refinement tracker item for GraphQL queries and mutations as complete


## Phase 69 - Opaque Definition Lint
- `lint_schema` now warns when exclusion constraints use raw SQL definitions that cannot be parsed.
- Added a new test verifying the warning is produced.
- Marked the refinement tracker item complete.

## Phase 70 - Database Backend Option
- Added a new `db_backend` field under `[meta]` with an enum supporting `postgres`, `mysql`, and `sqlite` (default `postgres`).
- `generate_code` selects pool types and SQL placeholders based on the backend, laying groundwork for future multi-database support.
- Updated the README and example schema to document the option.
- New test asserts the backend defaults to Postgres.

## Phase 71 - SQLite Introspection Support
- Refactored `introspect_schema` to accept a database URL and dispatch based on backend
- Added new helpers `introspect_schema_postgres`, `introspect_schema_sqlite`, and `introspect_schema_mysql`
- Implemented basic SQLite and MySQL introspection covering tables, columns, indexes, and foreign keys
- CLI commands and tests updated to pass URLs instead of pools
- This lays groundwork for multi-database functionality beyond Postgres

## Phase 72 - Generic Migration Application
- `apply_migrations` and `apply_seed_data` now accept `sqlx::AnyPool`, enabling execution against MySQL and SQLite.
- CLI `migrate apply` and `seed` commands connect using `AnyPool` based on the database URL.
- Updated tests to use `AnyPool` and enabled a new SQLite introspection round-trip test.

## Phase 73 - Structured Relation References
- `RelationDef` now stores a `FieldRef` with separate `model` and `field` names instead of a single string.
- Schema files define relations using `references = { model = "User", field = "id" }` syntax.
- Migration generation, linting, introspection, and GraphQL code all use the structured form.
- Updated example schema, README, and tests to the new format.

## Phase 74 - Backend-Specific Insert Logic
- `create` helper now adapts to the configured `db_backend`.
- For Postgres and SQLite, inserts use `RETURNING *` as before.
- For MySQL, the helper performs an INSERT followed by a `SELECT` using `LAST_INSERT_ID()` to return the new row.
- This begins broader multi-database support.

## Phase 75 - Backend-Specific Update Logic
- `update` helper now varies its SQL by backend.
- Postgres and SQLite continue using `RETURNING *` for updates.
- MySQL executes an `UPDATE` followed by a `SELECT` to fetch the updated row.
- Keeps CRUD helpers consistent across supported databases.

## Phase 76 - MySQL Introspection Enhancements
- `introspect_schema_mysql` now queries index metadata via `SHOW INDEX` and
  foreign keys using `information_schema.KEY_COLUMN_USAGE`.
- Captured indexes populate `IndexDef` with uniqueness, and foreign keys create
  `RelationDef` entries.
- README notes the expanded MySQL introspection support.

## Phase 77 - MySQL Migration Syntax
- `generate_initial_migration` and `generate_migration` now emit MySQL-compatible DROP statements for indexes, unique constraints, check constraints, and foreign keys.
- New helpers produce backend-aware SQL so migrations succeed across backends.

## Phase 78 - MySQL Constraint Introspection
- `introspect_schema_mysql` now reads unique and check constraints from `information_schema`.
- These constraints populate `UniqueConstraintDef` and `CheckConstraintDef` so diffs are accurate.
- The README discusses the expanded MySQL introspection.

## Phase 79 - SQLite Constraint Introspection
- `introspect_schema_sqlite` parses table definitions to capture named unique and
  check constraints.
- Unique constraints are detected from `PRAGMA index_list` when `origin` is `u`.
- Check constraint expressions are extracted using a regex over the `CREATE TABLE`
  statement.
- README mentions the new SQLite constraint support.

## Phase 80 - Basic Auth Configuration
- Meta now includes an `auth` table with `provider` and `anonymous_role` options.
- Code generation exposes an `ANONYMOUS_ROLE` constant and a `has_permission` helper.
- Routes and models remain unchanged but can leverage these helpers for RBAC checks.

## Phase 81 - Public Role Handling
- Introduced the `PUBLIC_ROLE` constant and updated `has_permission` to always allow this role.
- Linting now accepts the special `public` and configured anonymous role in permission lists.
- Example schema marks the `Post` route as publicly readable.

## Phase 82 - Auth Provider Enum
- Replaced the string-based auth provider with a new `AuthProvider` enum supporting `None` and `Jwt`.
- Updated documentation to show the available variants.

## Phase 83 - JWT Role Claim
- Added a `role_claim` option under `[meta.auth]` specifying which JWT claim stores the user's role.
- `AuthConfig` now includes this field and defaults to `role`.
- Code generation will expose a `ROLE_CLAIM` constant alongside helpers when JWT auth is enabled.
- Tests verify the new field parses correctly from `schema.model.toml`.


## Phase 84 - Route Permission Helpers
- Added `route_has_permission` in generated Rust code to check permissions per HTTP method.
- TypeScript output now includes matching `routeHasPermission` and permission interfaces.
- `load_type_aliases` improved error handling by removing `unwrap`.

## Phase 85 - Configurable Public Role
- `AuthConfig` includes a `public_role` field defaulting to `"public"`.
- Rust and TypeScript code generation emit `PUBLIC_ROLE` from this value.
- Linting accepts the configured `public_role` when validating permissions.
- README and the example schema document the new option.

## Phase 86 - Backend Detection
- Added `infer_backend_from_url` helper to parse the database backend from a connection URL.
- `introspect_schema` now uses this helper for dispatching.
- CLI commands rely on URL inference when connecting to databases.
- Tests cover backend detection logic.

## Phase 87 - Generic Connection Helper
- Introduced `connect_any_pool` which installs SQLx drivers and opens an `AnyPool` based on the URL.
- CLI commands for `migrate apply` and `seed` now use this helper.
- Added a new unit test verifying a SQLite in-memory URL connects successfully.

## Phase 88 - Typed Create Errors
- Generated `create` functions return `<Model>CreateError` enums using `thiserror`.
- Errors map unique and foreign key constraint names from the database to specific variants.
- Added tests verifying the enums and variants appear in generated code.

## Phase 89 - Hash-Based Migration Tracking
- The `__rustdbgen_migrations` table now records a SHA-256 hash, timestamp, execution time, and success flag for each migration.
- `apply_migrations` computes the hash of each file and refuses to run if a previously applied migration has been modified.
- Migrations execute inside a transaction and record their duration in milliseconds.
- README documents the new tracking behavior.


\n## Phase 90 - Paginated List Helpers
- Generated code defines a reusable `Pagination` struct.
- `list` helpers now accept `Option<Pagination>` and apply LIMIT/OFFSET via `QueryBuilder` for all backends.
- Tests ensure the pagination struct and signature are present in generated code.

## Phase 91 - Schema Macros
- Added a `[macros]` section allowing reusable field and option blocks.
- Models can declare `includes = ["macro"]` to merge macro fields and options.
- Implemented `apply_macros` to perform the merge before type aliases and options.
- Updated example models to include an `audit_fields` macro adding timestamp fields.
- Added tests verifying macro expansion.

## Phase 92 - Web Editor Prototype
- Added a `serve` CLI command launching a lightweight web UI for editing `schema.model.toml`.
- Implemented `serve_editor` in the library using Axum.
- Added `web/editor.html` and documentation.
