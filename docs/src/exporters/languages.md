# Programming languages

MD-Models can export your data models to various programming languages, generating type-safe code with validation, serialization, and additional features like JSON-LD support.

## Python

Python is a high-level, interpreted programming language known for its simplicity and readability. It's widely used in web development, data science, scientific computing, automation, and API development. Python's extensive ecosystem and ease of use make it ideal for rapid prototyping and production applications.

### Python Dataclass

The Python dataclass exporter generates Python classes using the `dataclasses` module and `dataclasses-json` for JSON serialization. This is ideal for simple data models that need basic validation and serialization.

**Features:**

- Type hints for all attributes
- Automatic JSON serialization/deserialization
- Runtime validation
- Helper methods for adding nested objects

**Usage:**

```bash
md-models convert -i <model> -t python-dataclass
```

**Example:**

```bash
md-models convert -i model.md -t python-dataclass -o models.py
```

**JSON-LD Support:** ✅ Yes - Includes `@id`, `@type`, and `@context` fields automatically

---

### Python Pydantic

The Python Pydantic exporter generates Pydantic models, which provide powerful runtime validation and type checking. Pydantic is widely used in modern Python applications, especially with FastAPI for building REST APIs, data validation in data pipelines, and configuration management.

**Features:**

- Runtime data validation
- Type coercion and conversion
- Field descriptions and documentation
- Filter methods for nested collections
- JSON-LD helper methods (`set_attr_term`, `add_type_term`)
- Support for Astropy units (via `astropy` option)

**Usage:**

```bash
md-models convert -i <model> -t python-pydantic
```

**Example:**

```bash
md-models convert -i model.md -t python-pydantic -o models.py
```

**Options:**

- `astropy`: Enable Astropy unit support for UnitDefinition types

  ```bash
  md-models convert -i model.md -t python-pydantic -O astropy -o models.py
  ```

**JSON-LD Support:** ✅ Yes - Includes JSON-LD fields and helper methods for managing semantic annotations

---

### Python Pydantic XML

The Python Pydantic XML exporter generates Pydantic models with XML serialization support using `pydantic-xml`. This is ideal for applications that need to work with XML data formats, such as SOAP services, legacy system integration, document processing, and scientific data exchange formats.

**Features:**

- XML serialization and deserialization
- Support for XML namespaces and attributes
- Wrapped XML elements
- Pretty-printed XML output
- Runtime validation

**Usage:**

```bash
md-models convert -i <model> -t python-pydantic-xml
```

**Example:**

```bash
md-models convert -i model.md -t python-pydantic-xml -o models.py
```

**JSON-LD Support:** ❌ No - Focused on XML serialization

---

## TypeScript

TypeScript is a typed superset of JavaScript that compiles to plain JavaScript. It adds static type checking to JavaScript, making it ideal for large-scale web applications, frontend frameworks (React, Vue, Angular), Node.js backend services, and anywhere type safety is crucial for maintainability and catching errors early.

### TypeScript (io-ts)

The TypeScript io-ts exporter generates TypeScript interfaces with runtime validation using the `io-ts` library. This provides both static type checking and runtime validation, making it perfect for API clients, data validation layers, and functional programming approaches in TypeScript.

**Features:**

- TypeScript interfaces with type inference
- Runtime validation with io-ts decoders
- Generic `validate` function for type-safe validation
- JSON-LD interface support

**Usage:**

```bash
md-models convert -i <model> -t typescript
```

**Example:**

```bash
md-models convert -i model.md -t typescript -o models.ts
```

**JSON-LD Support:** ✅ Yes - Includes `JsonLd` interface that all types extend

---

### TypeScript Zod

The TypeScript Zod exporter generates Zod schemas, which provide both runtime validation and TypeScript type inference. Zod is a popular choice for modern TypeScript applications, especially in form validation, API request/response validation, configuration schemas, and anywhere you need runtime type safety with excellent developer experience.

**Features:**

- Zod schema definitions with type inference
- Runtime validation
- Field descriptions
- Union type support
- Optional JSON-LD schema support

**Usage:**

```bash
md-models convert -i <model> -t typescript-zod
```

**Example:**

```bash
md-models convert -i model.md -t typescript-zod -o schemas.ts
```

**Options:**

- `json-ld`: Enable JSON-LD schema support

  ```bash
  md-models convert -i model.md -t typescript-zod -O json-ld -o schemas.ts
  ```

**JSON-LD Support:** ✅ Yes - Available via the `json-ld` option

---

## Rust

Rust is a systems programming language focused on safety, performance, and concurrency. It provides memory safety without garbage collection, making it ideal for systems programming, web servers, embedded systems, blockchain development, and performance-critical applications. Rust's strong type system and ownership model prevent many common programming errors at compile time.

The Rust exporter generates Rust struct definitions with serde serialization, builder pattern support, and optional JSON-LD functionality.

**Features:**

- Rust structs with serde serialization
- Builder pattern using `derive_builder`
- JSON Schema generation support (`schemars`)
- Optional JSON-LD header support
- Union types for multi-type attributes
- XML wrapped element support

**Usage:**

```bash
md-models convert -i <model> -t rust
```

**Example:**

```bash
md-models convert -i model.md -t rust -o models.rs
```

**Options:**

- `jsonld`: Enable JSON-LD header support with context management

  ```bash
  md-models convert -i model.md -t rust -O jsonld -o models.rs
  ```

When `jsonld` is enabled, the generated code includes:

- `JsonLdHeader` struct with context, ID, and type fields
- Helper methods for managing JSON-LD contexts (`add_term`, `update_term`, `remove_term`)
- Default JSON-LD header functions for each object type

**JSON-LD Support:** ✅ Yes - Available via the `jsonld` option

---

## Go

Go (Golang) is a statically typed, compiled language designed for simplicity, efficiency, and concurrency. Developed by Google, Go is widely used for building scalable backend services, microservices, cloud-native applications, command-line tools, and distributed systems. Its built-in concurrency primitives (goroutines and channels) make it excellent for handling high-throughput network services and concurrent operations.

The Go exporter generates Go struct definitions with JSON and XML serialization tags, and optional GORM support for database integration.

**Features:**

- Go structs with JSON/XML tags
- Automatic type conversion (string, float64, int64, bool, []byte)
- Union types for multi-type attributes with custom marshaling
- Optional GORM tags for database integration
- Self-referential type handling (pointers)

**Usage:**

```bash
md-models convert -i <model> -t golang
```

**Example:**

```bash
md-models convert -i model.md -t golang -o models.go
```

**Options:**

- `gorm`: Enable GORM tags for database relationships

  ```bash
  md-models convert -i model.md -t golang -O gorm -o models.go
  ```

- `xml`: Enable XML serialization tags

  ```bash
  md-models convert -i model.md -t golang -O xml -o models.go
  ```

When `gorm` is enabled, the generated code includes:

- Primary key tags (`gorm:"primaryKey"`)
- Foreign key relationships (`gorm:"foreignKey:..."`)
- Many-to-many relationships (`gorm:"many2many:..."`)
- JSON serializer tags for complex types (`gorm:"serializer:json"`)

When `xml` is enabled, the generated structs include XML tags for serialization, supporting:

- XML element names
- XML attributes
- Wrapped XML elements

**JSON-LD Support:** ❌ No

---

## Julia

Julia is a high-level, high-performance dynamic programming language designed for numerical and scientific computing. It combines the ease of use of Python with the performance of C, making it ideal for data science, machine learning, scientific simulations, computational biology, and high-performance numerical computing. Julia's multiple dispatch and just-in-time compilation enable both rapid prototyping and production performance.

The Julia exporter generates Julia struct definitions with JSON serialization support using JSON3 and StructTypes.

**Features:**

- Julia mutable structs with keyword constructors
- JSON serialization via JSON3
- Union types for optional and multi-type fields
- Type-safe field definitions
- Module-based organization

**Usage:**

```bash
md-models convert -i <model> -t julia
```

**Example:**

```bash
md-models convert -i model.md -t julia -o models.jl
```

**JSON-LD Support:** ❌ No

---

## Template Options

All templates support passing options via the `-O` or `--options` flag:

```bash
md-models convert -i <model> -t <template> -O option1,option2
```

Multiple options can be passed as a comma-separated list. Available options vary by template:

- **Python Pydantic**: `astropy`
- **TypeScript Zod**: `json-ld`
- **Rust**: `jsonld`
- **Go**: `gorm`, `xml`

Options are passed to the template as configuration flags and can modify the generated code structure and features.

---

## Language Comparison

| Language       | Primary Use Case                   | Runtime Validation | Type Safety | JSON-LD Support | Best For                                        |
| -------------- | ---------------------------------- | ------------------ | ----------- | --------------- | ----------------------------------------------- |
| **Python**     | Web APIs, Data Science             | ✅ Yes              | ✅ Yes       | ✅ Yes           | FastAPI, Data pipelines, Scientific computing   |
| **TypeScript** | Web Development, Frontend/Backend  | ✅ Yes              | ✅ Yes       | ✅ Yes           | React, Node.js, Type-safe APIs                  |
| **Rust**       | Systems Programming, Performance   | ✅ Yes              | ✅ Yes       | ✅ Yes           | Web servers, Embedded systems, High performance |
| **Go**         | Backend Services, Microservices    | ✅ Yes              | ✅ Yes       | ❌ No            | Cloud-native apps, Distributed systems          |
| **Julia**      | Scientific Computing, Data Science | ✅ Yes              | ✅ Yes       | ❌ No            | Numerical computing, Machine learning           |
