# RustDBGen

An experimental schema-driven code generator for Rust backends. Generate complete, production-ready APIs from TOML schema definitions with zero boilerplate. 

Current Status: Prototype.

## 🚀 Features

### **Core Code Generation**
- **Complete Rust Backend**: Generate full Axum-based REST APIs with CRUD operations
- **Database Models**: Type-safe Rust structs with Serde serialization
- **SQLx Integration**: Direct database queries with compile-time SQL validation
- **Multi-Database Support**: PostgreSQL, MySQL, and SQLite backends
- **Modular Architecture**: Clean separation of models, handlers, routes, and auth

### **Schema Definition**
- **TOML-Based**: Human-readable schema definition with includes and macros
- **Type Safety**: Strong typing with custom enum support
- **Field Validation**: Zod schema integration for runtime validation
- **Database Constraints**: Indexes, unique constraints, check constraints, foreign keys
- **Soft Deletes**: Built-in soft delete support with automatic filtering
- **Audit Fields**: Automatic timestamp fields (created_at, updated_at)

### **Authentication & Authorization**
- **JWT Authentication**: Built-in JWT token validation and claims extraction
- **Role-Based Access Control**: Fine-grained permissions per model and route
- **Anonymous Access**: Configurable guest/anonymous user support
- **Public Routes**: Special role for publicly accessible endpoints
- **Ownership Model**: Automatic user ownership filtering for multi-tenant apps

### **API Features**
- **RESTful Endpoints**: Standard CRUD operations (GET, POST, PUT, DELETE)
- **Pagination**: Built-in pagination support for list operations
- **Error Handling**: Typed error responses with proper HTTP status codes
- **CORS Support**: Configurable CORS middleware
- **Health Checks**: Built-in health check endpoint

### **Database Management**
- **Migration Generation**: Automatic SQL migration files from schema changes
- **Schema Introspection**: Reverse-engineer schemas from existing databases
- **Migration Validation**: Check for pending migrations without applying
- **Seed Data**: Generate and apply seed data for development
- **Transaction Support**: Full transaction support in generated code

### **Developer Experience**
- **Observability**: Built-in tracing support with structured logging
- **TypeScript Generation**: Generate TypeScript interfaces and client code
- **GraphQL Support**: Generate GraphQL schemas from your models
- **Plugin System**: Extensible plugin architecture for custom generators
- **Configuration Management**: Environment-based configuration with validation

### **Advanced Features**
- **File Storage**: S3 and local file storage integration
- **Password Hashing**: Automatic Argon2 password hashing
- **Search Support**: Built-in search field tagging
- **Custom Validations**: Zod schema integration for field validation
- **Executor Pattern**: Flexible database executor trait for transactions

## 📋 Quick Start

### 1. Define Your Schema

Create a `schema.model.toml` file:

```toml
schema_version = "1.0"

[meta]
db_backend = "postgres"
observability_provider = "tracing"

[meta.auth]
provider = "jwt"
anonymous_role = "guest"
role_claim = "role"
public_role = "public"

[enums.Role]
variants = ["admin", "member", "guest"]

[models.User]
fields.id = { type = "Uuid", db_type = "UUID PRIMARY KEY", default = "gen_random_uuid()" }
fields.email = { type = "String", db_type = "VARCHAR(255)", nullable = false, zod = "z.string().email()" }
fields.password_hash = { type = "String", db_type = "VARCHAR(255)", nullable = false, tags = ["password"] }
fields.role = { type = "Role", db_type = "role", default = "'member'" }

[models.User.options]
soft_delete = true

[models.User.permissions]
read = ["admin", "member"]
update = ["admin", "member"]
delete = ["admin"]

[routes.User]
methods = ["GET", "POST", "PUT", "DELETE"]
path = "/api/users"
auth_required = true

[routes.User.permissions]
read = ["admin", "member"]
update = ["admin", "member"]
delete = ["admin"]
```

### 2. Generate Your Backend

```bash
# Generate complete backend
rustdbgen generate

# Generate migrations
rustdbgen migrate generate initial

# Apply migrations
rustdbgen migrate apply

# Generate TypeScript types
rustdbgen generate-ts

# Generate GraphQL schema
rustdbgen generate-graphql
```

### 3. Run Your API

```bash
cd backend
cargo run
```

Your API is now running with full CRUD operations, authentication, and authorization!

## 🏗️ Architecture

### Generated Structure

```
backend/
├── src/
│   ├── main.rs                 # Application entry point
│   └── generated/
│       ├── models/             # Database models and structs
│       ├── handlers/           # CRUD operation implementations
│       ├── routes/             # Route definitions and permissions
│       ├── router.rs           # Axum router configuration
│       ├── auth.rs             # JWT authentication middleware
│       ├── permissions.rs      # Role-based access control
│       ├── pagination.rs       # Pagination utilities
│       ├── executor.rs         # Database executor traits
│       ├── config.rs           # Configuration management
│       └── main.rs             # Server setup and configuration
├── migrations/                 # SQL migration files
└── Cargo.toml                 # Generated dependencies
```

### Key Components

- **Models**: Type-safe Rust structs with Serde serialization
- **Handlers**: Database operations with proper error handling
- **Router**: RESTful API endpoints with authentication
- **Auth**: JWT middleware with role-based permissions
- **Executor**: Database abstraction for transactions
- **Config**: Environment-based configuration

## 🔧 Configuration

### Database Backends

```toml
[meta]
db_backend = "postgres"  # postgres, mysql, sqlite
```

### Authentication

```toml
[meta.auth]
provider = "jwt"           # jwt, none
anonymous_role = "guest"   # role for unauthenticated users
role_claim = "role"        # JWT claim for user role
public_role = "public"     # role granted to anyone
```

### Observability

```toml
[meta]
observability_provider = "tracing"  # Enable structured logging
```

## 📚 Schema Features

### Macros

Reusable field definitions:

```toml
[macros.audit_fields.fields.created_at]
type = "DateTime<Utc>"
db_type = "TIMESTAMPTZ"
default = "now()"

[macros.audit_fields.fields.updated_at]
type = "DateTime<Utc>"
db_type = "TIMESTAMPTZ"
default = "now()"

[models.User]
includes = ["audit_fields"]
```

### Relations

```toml
[models.Post]
fields.author_id = { type = "Uuid", db_type = "UUID" }
relations.author = { on = "author_id", references = { model = "User", field = "id" } }
```

### Constraints

```toml
[models.User]
indexes.email_unique = { fields = ["email"], unique = true }
check_constraints.email_not_empty = { expression = "email <> ''" }
```

### File Storage

```toml
[models.User]
fields.profile_pic = { 
    type = "File", 
    storage = { 
        backend = "s3", 
        allowed_types = ["image/png", "image/jpeg"], 
        path = "uploads/users/{id}/" 
    } 
}
```

## 🔌 Plugin System

Extend functionality with custom plugins:

```toml
[plugins.my_generator]
command = "my-generator"
args = ["--config", "custom.toml"]
env.CUSTOM_VAR = "value"
```

## 🚀 CLI Commands

```bash
# Generate backend code
rustdbgen generate

# Generate migrations
rustdbgen migrate generate <name>
rustdbgen migrate check
rustdbgen migrate apply

# Generate client code
rustdbgen generate-ts
rustdbgen generate-graphql

# Schema validation
rustdbgen lint

# Database introspection
rustdbgen introspect <database_url>
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Test specific features
cargo test --test generate
cargo test --test migration
cargo test --test lint
```

## 📦 Dependencies

The generated backend includes:

- **Axum**: Web framework
- **SQLx**: Database toolkit
- **Serde**: Serialization
- **JWT**: Authentication
- **Tracing**: Observability
- **Tower**: HTTP middleware
- **Chrono**: Date/time handling
- **UUID**: Unique identifiers

## 🎯 Use Cases

- **REST APIs**: Complete backend APIs with authentication
- **Microservices**: Modular, scalable service architecture
- **Multi-tenant Apps**: Built-in ownership and role-based access
- **Admin Panels**: Full CRUD operations with permissions
- **Mobile Backends**: JSON APIs with proper error handling
- **Internal Tools**: Rapid prototyping and development

## 🔒 Security Features

- **JWT Authentication**: Secure token-based authentication
- **Role-Based Access**: Fine-grained permission control
- **Password Hashing**: Argon2 password security
- **Input Validation**: Zod schema validation
- **SQL Injection Protection**: SQLx compile-time query validation
- **CORS Configuration**: Configurable cross-origin policies

## 📈 Performance

- **Compile-Time SQL**: SQLx ensures query correctness at compile time
- **Connection Pooling**: Efficient database connection management
- **Async/Await**: Non-blocking I/O throughout
- **Structured Logging**: Efficient observability with tracing
- **Type Safety**: Zero runtime type errors

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details.

---

**RustDBGen**: From schema to production-ready API in minutes, not days.
