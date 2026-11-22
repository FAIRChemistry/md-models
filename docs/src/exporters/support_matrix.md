# Feature-to-target support matrix

Which semantic features of the Markdown format are carried into each target, and
how. Reviewers of the manuscript asked for this twice — once as a
"feature/validation coverage matrix" and once as a "feature-to-target mapping
table" — because a reader who already knows JSON Schema, SHACL, Pydantic or GORM
needs to locate this format against something they understand, and see what they
gain and what they give up.

The cells below were checked against generated output for `mdmodels 0.2.10`.
The rows on run-time type checks, validation options and primary keys in the
programming-language table were re-checked for `mdmodels 0.3.1`.

The cross-target conformance harness (`tests/conformance/validate_artifacts.py`)
checks something narrower: that every artefact is well formed, meaning it is
accepted by that ecosystem's own tool. It does not compare targets with each
other, and it does not test the cells of this table. All targets share one
structure because they are generated from one parsed model; they differ in the
value constraints they can carry, which is what the tables below record.

**Legend**

| Symbol | Meaning |
|:--:|---|
| ● | Fully expressed in the target's native construct |
| ◐ | Partially expressed — see the note |
| ○ | Not expressible in this target; silently absent from the output |
| – | Not applicable to this target |

---

## Schema languages

| Feature | JSON Schema | XSD | SHACL | ShEx | OWL | LinkML | JSON-LD |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| Objects | ● | ● | ● | ● | ● | ● | ● |
| Composition (object-typed attributes) | ● | ● | ● | ● | ● | ● | ● |
| Recursive / self-referencing objects | ● | ● | ● | ● | ● | ● | ◐ ¹ |
| Required vs optional | ● | ● | ● | ● | ● | ● | – |
| Multivalued attributes | ● | ● | ● | ● | ● | ● | ● |
| Union types (`Type: a, b`) | ● `oneOf` | ● `xs:union` ² | ◐ ³ | ◐ ³ | ◐ ³ | ● `any_of` | – |
| Enumerations | ● | ● | ● | ● | ● | ● | – |
| Descriptions | ● | ● | ● | ● | ● | ● | – |
| Ontology terms (`Term:`) | ◐ | ○ | ● | ● | ● | ● | ● |
| XML naming (`XML:`) | ○ | ● | ○ | ○ | ○ | ○ | ○ |
| Default values | ● | ● | ○ | ○ | ○ | ○ | ○ |
| Primary key (`PK:`) | ○ | ○ | ○ | ○ | ○ | ○ | ○ |

**Validation options**

| Option | JSON Schema | XSD | SHACL | ShEx | OWL | LinkML |
|---|:--:|:--:|:--:|:--:|:--:|:--:|
| `Minimum` / `Maximum` | ● | ● `minInclusive`/`maxInclusive` | ○ | ○ | ○ | ● `minimum_value`/`maximum_value` |
| `ExclusiveMinimum` / `ExclusiveMaximum` | ● | ● `minExclusive`/`maxExclusive` | ○ | ○ | ○ | ○ |
| `MinLength` / `MaxLength` | ● | ● `minLength`/`maxLength` | ○ | ○ | ○ | ○ |
| `Pattern` / `Regex` | ● | ● `xs:pattern` ⁴ | ○ | ○ | ○ | ● `slot_usage.pattern` |
| `MinItems` / `MaxItems` | ● | ◐ `minOccurs`/`maxOccurs` | ◐ `sh:minCount` | ○ | ◐ cardinality | ○ |
| `MultipleOf` | ● | ○ ⁵ | ○ | ○ | ○ | ○ |
| `Unique` | ● | ○ | ○ | ○ | ○ | ○ |
| `Example` | ● | ○ | ○ | ○ | ○ | ● `examples` |
| `ReadOnly` / `Recommended` | ○ | ○ | ○ | ○ | ○ | ○ ⁶ |

## Programming languages

| Feature | Rust | Python (Pydantic) | Python (dataclass) | TypeScript | TypeScript (Zod) | Go | Julia |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| Objects | ● | ● | ● | ● | ● | ● | ● |
| Composition | ● | ● | ● | ● | ● | ● | ● |
| Required vs optional | ● | ● | ● | ● | ● | ● | ● |
| Multivalued attributes | ● | ● | ● | ● | ● | ● | ● |
| Union types | ● `enum` | ● | ● | ● | ● | ◐ | ● |
| Enumerations | ● | ● | ● | ● | ● | ● | ● |
| Descriptions | ● | ● | ● | ● | ● | ● | ● |
| Ontology terms | ● | ● | ○ | ○ | ● | ○ | ○ |
| Types checked at run time ⁷ | ● | ● | ○ | ○ | ● | ● | ● |
| Validation options enforced ⁷ | ○ | ◐ `pattern` only | ○ | ○ | ○ | ○ | ○ |
| Primary key (`PK:`) | ○ | ○ | ○ | ○ | ○ | ◐ with `-O gorm` ⁹ | ○ |

## Other targets

| Feature | Protobuf | GraphQL | Mermaid | MkDocs | Markdown |
|---|:--:|:--:|:--:|:--:|:--:|
| Objects | ● | ● | ● | ● | ● |
| Composition | ● | ● | ● | ● | ● |
| Required vs optional | ● | ● | ○ | ● | ● |
| Multivalued attributes | ● | ● | ● | ● | ● |
| Union types | ● wrapped `oneof` | ○ ⁸ | ○ | ● | ● |
| Enumerations | ● | ● | ● | ● | ● |
| Validation options | ○ | ○ | ○ | ● | ● |

---

## Notes

1. **JSON-LD and recursion.** A cycle is broken by omitting the nested
   `@context` at the point of re-entry. This is valid JSON-LD — scoped contexts
   are inherited from the enclosing context — but the nested definition is not
   repeated. Before v0.3.1 a recursive model aborted the process with a stack
   overflow.
2. **XSD unions.** `xs:union` accepts simple types only. A union whose members
   include an object (a `complexType`) cannot be expressed and falls back to the
   first member type. A union of primitives and/or enumerations is expressed
   exactly.
3. **Unions in RDF.** SHACL, ShEx and OWL receive the first declared type. RDF
   shape languages can express alternatives (`sh:or`), but the generated shapes
   do not currently do so for datatype unions.
4. **XSD patterns.** An `xs:pattern` is implicitly anchored to the whole value
   and treats `^` and `$` as ordinary characters, so leading `^` and trailing `$`
   are stripped on export. A pattern relying on other PCRE-only constructs may
   still behave differently from the JSON Schema one.
5. **`MultipleOf` in XSD.** XML Schema has no divisibility facet. The constraint
   is dropped, and this is the reason the matrix exists: a dropped constraint
   should be visible in a table rather than discovered in production.
6. **`ReadOnly` / `Recommended` in LinkML.** LinkML defines both slots and the
   exporter has the fields, but they are not currently populated.
7. **Run-time checks in generated code.** Two different checks are easy to
   conflate. *Types* are checked by every target except the Python dataclass and
   the plain TypeScript interface: at compile time in Rust, Go and TypeScript, and
   at run time, when a record is constructed or deserialised, by Pydantic, Zod,
   Rust (serde), Go (`encoding/json`) and Julia. TypeScript types are erased at run
   time, and dataclass type hints are not checked. *Validation options*
   (`Minimum`, `MaxLength`, `MinItems`, …) are value constraints that these types
   cannot express. Of the generated code, only Pydantic enforces one of them,
   `pattern`; bounds, lengths and item counts are enforced by no generated
   library. To check a dataset against every option, validate it against the
   generated JSON Schema, which carries all of them:

   ```bash
   md-models dataset validate -i data.json -m model.md
   ```

   Each violation is reported with its JSON path.
8. **GraphQL unions.** GraphQL unions may only combine object types, so a union
   of scalars is not expressible; the first type is used.
9. **GORM primary keys.** The Go generator writes GORM tags only with
   `-O gorm`. A `PK:` attribute is then tagged `primaryKey`, but the generated
   `Id` field keeps its own `primaryKey;autoIncrement` tag, so the struct has two
   primary-key fields.

## Targets that do not exist

`avro` is named in some documentation but is **not** an available target. Run
`md-models convert --help` for the authoritative list.

## Per-target maturity

| Check in CI | Targets | Basis |
|---|---|---|
| Well-formedness, with the ecosystem's own tool | `json-schema`, `xml-schema`, `shacl`, `owl`, `protobuf`, `graphql`, `linkml`; `json-ld` is checked to be valid JSON | Five reference models, validated with `jsonschema`, `xmllint`, `rdflib`, `pyshacl`, `protoc`, `graphql-core`, `linkml-runtime` |
| Generated code compiled and run | `rust`, `python-pydantic`, `python-dataclass`, `python-pydantic-xml`, `typescript-zod`, `golang`, `julia` | Against `tests/data/model.md`; `python-pydantic-xml` also against `tests/data/model_required.md` |
| Compared with a checked-in reference file | `typescript`, `mermaid`, `mkdocs`, `markdown` | Output must match the reference exactly |
| Non-empty only | `shex` | No Rust-side ShEx validator is available |

Running the generated code against a reference model shows that it works for
that model, not for every model. Before v0.3.1, for example, `python-pydantic-xml`
produced a module that failed to import when a model had a required attribute
without a default; the reference model had no such attribute.

## Running the conformance checks

The harness checks well-formedness only (see above). To check the cells of the
tables, generate the target and inspect its output.

```bash
cargo build --release
python3 tests/conformance/validate_artifacts.py --model tests/data/model_recursive.md
```

Add `--strict` to fail when a validator is missing rather than skipping it; that
is the mode CI should use. Optional validators: `rdflib`, `jsonschema`,
`PyYAML`, `linkml-runtime`, `graphql-core` (pip) and `protoc`, `xmllint`
(system).
