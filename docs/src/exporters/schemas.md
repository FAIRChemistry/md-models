# Schema languages

MD-Models can export your data models to various schema languages, enabling validation, semantic annotation, and interoperability across different systems and standards.

## JSON Schema

JSON Schema is a vocabulary that allows you to annotate and validate JSON documents. It provides a way to describe the structure of JSON data, making it ideal for API documentation, data validation, form generation, and ensuring data consistency across systems.

The JSON Schema exporter generates JSON Schema Draft 2020-12 compliant schemas from your MD-Models data model. These schemas can be used to validate JSON data, generate API documentation, and provide type information for various tools and frameworks.

**Usage:**

```bash
md-models convert -i <model> -t json-schema -r <root_object>
```

**Example:**

```bash
md-models convert -i model.md -t json-schema -r MyObject -o schema.json
```

**Options:**

- `openai`: Remove options from schema properties (OpenAI function calling compatibility)
  ```bash
  md-models convert -i model.md -t json-schema -r MyObject -O openai -o schema.json
  ```

**Features:**

- Generates JSON Schema Draft 2020-12 compliant schemas
- Includes all referenced types in `$defs` section
- Supports required fields, default values, and constraints
- Handles nested objects and enumerations
- Optional OpenAI-compatible mode

**Note:** The `-r` or `--root` parameter is required to specify which object should be the root of the schema.

---

## JSON Schema All

The JSON Schema All exporter generates separate JSON Schema files for each object in your data model. This is useful when you need individual schemas for each type rather than a single schema with definitions.

**Usage:**

```bash
md-models convert -i <model> -t json-schema-all -o <output_directory>
```

**Example:**

```bash
md-models convert -i model.md -t json-schema-all -o schemas/
```

**Features:**

- Generates one JSON Schema file per object type
- Each file is named after the object (e.g., `MyObject.json`)
- Useful for API documentation where each endpoint has its own schema
- Output directory is required (will be created if it doesn't exist)

---

## XML Schema (XSD)

XML Schema Definition (XSD) is a W3C standard for describing and validating the structure and content of XML documents. XSD is widely used in enterprise systems, SOAP web services, document exchange, and legacy system integration.

The XML Schema exporter generates W3C XML Schema 1.1 compliant schemas from your MD-Models data model. These schemas define the structure, data types, and constraints for XML documents.

**Usage:**

```bash
md-models convert -i <model> -t xml-schema
```

**Example:**

```bash
md-models convert -i model.md -t xml-schema -o schema.xsd
```

**Features:**

- Generates W3C XML Schema 1.1 compliant schemas
- Supports XML attributes and elements
- Handles required/optional fields and cardinality (minOccurs/maxOccurs)
- Supports default values
- Generates complex types for nested objects
- Supports XML namespace declarations

**Generated Output:**

- Complex type definitions for each object
- Simple type definitions for enumerations
- Element and attribute declarations
- Namespace prefixes and target namespaces

---

## SHACL

SHACL (Shapes Constraint Language) is a W3C standard for validating RDF data graphs. SHACL shapes describe constraints that RDF data must satisfy, making it ideal for data quality validation, semantic web applications, and linked data validation.

The SHACL exporter generates SHACL shapes in Turtle (TTL) format from your MD-Models data model. These shapes can be used with SHACL validators to check RDF data against your model constraints.

**Usage:**

```bash
md-models convert -i <model> -t shacl
```

**Example:**

```bash
md-models convert -i model.md -t shacl -o shapes.ttl
```

**Features:**

- Generates SHACL NodeShapes for each object type
- Property shapes with cardinality constraints (sh:minCount, sh:maxCount)
- Datatype constraints (sh:datatype)
- Enumeration constraints (sh:in)
- Class constraints (sh:class)
- Term-based property paths (supports semantic annotations)
- Class-scoped prefixes for semantic web integration

**Requirements:**

- Objects must have semantic terms defined (via `@term` annotations)
- The model must include ontology prefixes for proper RDF namespace handling

**Use Cases:**

- Validating RDF/JSON-LD data against semantic models
- Data quality assurance in knowledge graphs
- Semantic web application development
- Linked data validation pipelines

---

## ShEx

ShEx (Shape Expressions) is a language for describing and validating RDF graphs. Similar to SHACL, ShEx provides a concise syntax for expressing constraints on RDF data, making it popular in bioinformatics, semantic web applications, and RDF validation tools.

The ShEx exporter generates ShEx shape expressions from your MD-Models data model. ShEx provides a more compact syntax than SHACL and is well-suited for validating RDF data in various formats.

**Usage:**

```bash
md-models convert -i <model> -t shex
```

**Example:**

```bash
md-models convert -i model.md -t shex -o shapes.shex
```

**Features:**

- Generates ShEx shape definitions for each object type
- Cardinality constraints (`?`, `*`, `+`)
- Datatype constraints
- Enumeration constraints
- Class constraints
- Term-based property paths
- Class-scoped prefixes

**Requirements:**

- Objects must have semantic terms defined (via `- Term: <term>` annotations)
- The model must include ontology prefixes for proper RDF namespace handling

**Use Cases:**

- RDF data validation
- Semantic web applications
- Bioinformatics data validation
- Knowledge graph quality assurance

---

## OWL

OWL (Web Ontology Language) is a W3C standard for defining ontologies on the semantic web. OWL allows you to express rich logical relationships between concepts, making it ideal for knowledge representation, semantic reasoning, and building sophisticated knowledge graphs.

The OWL exporter generates OWL 2 ontologies in Turtle (TTL) format from your MD-Models data model. These ontologies define classes, properties, and their relationships in a machine-readable format suitable for semantic reasoning and inference.

**Usage:**

```bash
md-models convert -i <model> -t owl
```

**Example:**

```bash
md-models convert -i model.md -t owl -o ontology.ttl
```

**Features:**

- Generates OWL 2 ontology definitions
- Class definitions with rdfs:comment descriptions
- Object properties for relationships between classes
- Datatype properties for primitive attributes
- Enumeration classes for enum types
- Subclass relationships (rdfs:subClassOf)
- Property domain and range constraints
- Ontology metadata and prefixes

**Generated Output:**

- OWL ontology header with metadata
- Class definitions for each object type
- Object properties for relationships
- Datatype properties for attributes
- Enumeration class definitions
- Property constraints (domain, range, cardinality)

**Use Cases:**

- Building semantic knowledge bases
- Semantic reasoning and inference
- Knowledge graph construction
- Semantic web application development
- Ontology-driven data integration

---

## JSON-LD

JSON-LD (JSON for Linked Data) is a method of encoding linked data using JSON. It provides a way to add semantic meaning to JSON data through context definitions, making JSON data part of the semantic web while maintaining compatibility with existing JSON tools.

The JSON-LD exporter generates JSON-LD context headers (`@context`, `@id`, `@type`) from your MD-Models data model. These headers provide the semantic context needed to interpret JSON data as linked data.

**Usage:**

```bash
md-models convert -i <model> -t json-ld -r <root_object>
```

**Example:**

```bash
md-models convert -i model.md -t json-ld -r MyObject -o context.jsonld
```

**Features:**

- Generates JSON-LD `@context` with term definitions
- Includes `@id` and `@type` for the root object
- Maps model attributes to semantic terms
- Supports nested contexts for complex object graphs
- Includes ontology prefixes and namespace mappings

**Note:** The `-r` or `--root` parameter is optional. If not specified, the first object in the model is used as the root.

**Use Cases:**

- Adding semantic annotations to JSON APIs
- Creating linked data from structured JSON
- Semantic web data serialization
- Knowledge graph data exchange
- Schema.org and other vocabulary integration

---

## LinkML

LinkML (Linked Data Modeling Language) is a modeling language for building schemas that can be used to generate various artifacts including JSON Schema, Python classes, RDF, and more. LinkML provides a YAML-based schema format that bridges the gap between data modeling and code generation.

The LinkML exporter generates LinkML YAML schemas from your MD-Models data model. These schemas can be used with the LinkML toolkit to generate code, documentation, and other artifacts in multiple formats.

**Usage:**

```bash
md-models convert -i <model> -t linkml -o schema.yaml
```

**Example:**

```bash
md-models convert -i model.md -t linkml -o schema.yaml
```

**Features:**

- Generates LinkML YAML schema format
- Class definitions with attributes
- Slot definitions (shared attributes across classes)
- Enumeration definitions
- Prefix and import declarations
- Tree root identification
- Dependency-aware class ordering

**Generated Output:**

- LinkML schema header with ID, name, and prefixes
- Class definitions with slot usage
- Global slot definitions
- Enumeration definitions with permissible values
- Import declarations for external schemas

**Use Cases:**

- Multi-format code generation via LinkML toolkit
- Schema-driven development workflows
- Interoperability between different schema formats
- Biomedical and scientific data modeling
- Building comprehensive data model ecosystems

---

## Schema Comparison

| Schema Format   | Primary Use Case        | Output Format | Validation Target |
| --------------- | ----------------------- | ------------- | ----------------- |
| **JSON Schema** | JSON data validation    | JSON          | JSON documents    |
| **XML Schema**  | XML document validation | XML (XSD)     | XML documents     |
| **SHACL**       | RDF graph validation    | Turtle (TTL)  | RDF graphs        |
| **ShEx**        | RDF graph validation    | ShEx          | RDF graphs        |
| **OWL**         | Ontology definition     | Turtle (TTL)  | Knowledge graphs  |
| **JSON-LD**     | Linked data context     | JSON          | JSON-LD documents |
| **LinkML**      | Multi-format schema     | YAML          | Various formats   |

Each schema format serves different purposes in the data modeling ecosystem, from runtime validation (JSON Schema, XML Schema) to semantic web technologies (SHACL, ShEx, OWL) to multi-format code generation (LinkML).
