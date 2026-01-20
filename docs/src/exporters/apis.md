# API specifications

MD-Models can export your data models to API specification formats, enabling you to generate type-safe API schemas and client/server code.

## GraphQL

The GraphQL exporter generates GraphQL Schema Definition Language (SDL) files from your MD-Models data model. This allows you to:

- Define GraphQL types, enums, and queries based on your data model
- Generate type-safe GraphQL APIs
- Use union types for attributes with multiple possible types
- Automatically create query types for fetching collections and filtering by attributes

### Usage

To generate a GraphQL schema from your MD-Models file:

```bash
md-models convert -i <model> -t graphql
```

For example:

```bash
md-models convert -i model.md -t graphql -o schema.graphql
```

### Generated Output

The GraphQL exporter generates:

- **Type definitions**: Each object in your model becomes a GraphQL type with its attributes as fields
- **Union types**: Attributes with multiple possible types are converted to GraphQL union types
- **Enum definitions**: Enumerations from your model are converted to GraphQL enums
- **Query type**: Automatically generates query operations including:
  - `all{ObjectName}s`: Query to fetch all instances of an object type
  - `{objectName}By{AttributeName}`: Query to filter objects by scalar attribute values

The exporter automatically maps MD-Models types to GraphQL scalar types:

- `integer` → `Int`
- `float` / `number` → `Float`
- `boolean` → `Boolean`
- `string` → `String`
- `bytes` → `String`
- `date` → `String`

Fields are marked as required (using `!`) based on the `required` attribute in your model, and arrays are represented using GraphQL list syntax `[Type]`.

## Protobuf

The Protobuf exporter generates Protocol Buffer (protobuf) message definitions from your MD-Models data model. Protocol Buffers are Google's language-neutral, platform-neutral mechanism for serializing structured data, commonly used for:

- Efficient data serialization in microservices
- Cross-language data exchange
- gRPC service definitions
- High-performance data storage and transmission

### Usage

To generate a Protocol Buffer schema from your MD-Models file:

```bash
md-models convert -i <model> -t protobuf
```

For example:

```bash
md-models convert -i model.md -t protobuf -o schema.proto
```

### Generated Output

The Protobuf exporter generates:

- **Message definitions**: Each object in your model becomes a protobuf message type
- **Enum definitions**: Enumerations from your model are converted to protobuf enums
- **OneOf types**: Attributes with multiple possible types are converted to protobuf `oneof` fields
- **Field rules**: Fields are marked as `repeated` for arrays or `optional` for non-required fields
- **Package declaration**: Uses the model title (or "model" if not specified) as the package name

The exporter uses proto3 syntax and automatically maps MD-Models types to protobuf types:

- `string` → `string`
- `float` → `double`
- `int` → `int32`
- `bool` → `bool`
- Object types and enums are preserved as-is

Each field is assigned a unique field number starting from 1, following protobuf conventions.

---

## API Specification Comparison

| Format       | Primary Use Case             | Serialization Format | Type System   | Best For                                   |
| ------------ | ---------------------------- | -------------------- | ------------- | ------------------------------------------ |
| **GraphQL**  | Flexible API queries         | JSON                 | Strong typing | Modern web APIs, Flexible data fetching    |
| **Protobuf** | Efficient data serialization | Binary/Text          | Strong typing | Microservices, gRPC, High-performance APIs |
