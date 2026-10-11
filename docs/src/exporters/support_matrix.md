# Feature-to-target support matrix

Which semantic features of the Markdown format are carried into each target, and
how. Reviewers of the manuscript asked for this twice — once as a
"feature/validation coverage matrix" and once as a "feature-to-target mapping
table" — because a reader who already knows JSON Schema, SHACL, Pydantic or GORM
needs to locate this format against something they understand, and see what they
gain and what they give up.

Every cell below was verified against generated output for
`mdmodels 0.2.10`. The cross-target conformance harness
(`tests/conformance/validate_artifacts.py`) regenerates every artefact and
validates it with that ecosystem's own tool, so the claims here are checkable
rather than asserted.

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
| **Runtime validation** | ○ ⁷ | ● | ○ ⁷ | ○ ⁷ | ● | ○ ⁷ | ○ ⁷ |
| Validation options enforced | ○ | ● | ○ | ○ | ● | ○ | ○ |

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
   repeated. Before v0.2.11 a recursive model aborted the process with a stack
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
7. **"Runtime validation" in generated code.** Only the Pydantic and Zod targets
   validate at runtime. Rust structs, Go structs, Python dataclasses and plain
   TypeScript interfaces are *typed*, and TypeScript types are erased entirely at
   runtime; none of them enforce the validation options. The distinction matters
   when reading claims about generated libraries "providing validation logic".
8. **GraphQL unions.** GraphQL unions may only combine object types, so a union
   of scalars is not expressible; the first type is used.

## Targets that do not exist

`avro` is named in some documentation but is **not** an available target. Run
`md-models convert --help` for the authoritative list.

## Per-target maturity

| Maturity | Targets | Basis |
|---|---|---|
| Covered by conformance checks | `json-schema`, `json-ld`, `shacl`, `owl`, `xml-schema`, `protobuf`, `linkml` | Validated on every run with `jsonschema`, `rdflib`, `xmllint`, `protoc` |
| Covered by golden-file tests | `rust`, `python-pydantic`, `python-dataclass`, `typescript`, `typescript-zod`, `golang`, `julia`, `graphql`, `mermaid`, `mkdocs`, `markdown` | Output compared against a checked-in reference |
| Structural checks only | `shex` | No Rust-side ShEx validator is available |

## Reproducing this table

```bash
cargo build --release
python3 tests/conformance/validate_artifacts.py --model tests/data/model_recursive.md
```

Add `--strict` to fail when a validator is missing rather than skipping it; that
is the mode CI should use. Optional validators: `rdflib`, `jsonschema`,
`PyYAML`, `linkml-runtime`, `graphql-core` (pip) and `protoc`, `xmllint`
(system).
