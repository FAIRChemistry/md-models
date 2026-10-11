//! Property-based tests asserting invariants over generated inputs.
//!
//! Generators are biased toward the shapes of the Markdown dialect, since
//! uniform random bytes rarely reach an interesting parser state. Set
//! `PROPTEST_CASES` to widen the search.

use std::collections::HashSet;

use mdmodels_core::prelude::{DataModel, Templates};
use proptest::prelude::*;

/// Every export target, so a new one cannot be added without being covered.
const ALL_TEMPLATES: &[Templates] = &[
    Templates::XmlSchema,
    Templates::Markdown,
    Templates::CompactMarkdown,
    Templates::JsonSchema,
    Templates::JsonLd,
    Templates::Shacl,
    Templates::Owl,
    Templates::Shex,
    Templates::PythonDataclass,
    Templates::PythonPydanticXML,
    Templates::PythonPydantic,
    Templates::MkDocs,
    Templates::Internal,
    Templates::Typescript,
    Templates::TypescriptZod,
    Templates::Rust,
    Templates::Protobuf,
    Templates::Graphql,
    Templates::Golang,
    Templates::Linkml,
    Templates::Julia,
    Templates::Mermaid,
];

//
// Generators
//

/// Identifier-ish names, including ones that collide with target keywords.
fn arb_name() -> impl Strategy<Value = String> {
    prop_oneof![
        8 => "[A-Za-z][A-Za-z0-9_]{0,12}",
        1 => Just("Any".to_string()),          // collides with the LinkML union placeholder
        1 => Just("Query".to_string()),        // collides with the GraphQL root type
        1 => prop::sample::select(vec![
            "type", "class", "enum", "message", "package", "import", "struct", "interface",
        ]).prop_map(String::from),
    ]
}

/// Primitive types plus the spellings that are not primitives.
fn arb_primitive() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "string", "integer", "float", "number", "boolean", "bytes", "date",
    ])
    .prop_map(String::from)
}

/// Option lines covering the literal values a target has to carry through.
fn arb_option_line() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("  - Description: a description".to_string()),
        Just("  - Term: schema:name".to_string()),
        Just("  - Term: dcterms:created".to_string()),
        Just("  - Term: https://example.org/bare".to_string()),
        "  - Pattern: \\^\\[A-Za-z0-9 _-\\]\\+\\$".prop_map(String::from),
        Just("  - Pattern: ^[a-z _]+$".to_string()),
        Just("  - Pattern: ^a _b_ c$".to_string()),
        Just("  - Pattern: [a-z]+".to_string()),
        (0i64..100).prop_map(|n| format!("  - Minimum: {n}")),
        (100i64..1000).prop_map(|n| format!("  - Maximum: {n}")),
        (1usize..10).prop_map(|n| format!("  - MinLength: {n}")),
        (10usize..99).prop_map(|n| format!("  - MaxLength: {n}")),
        Just("  - Unique: true".to_string()),
        Just("  - PK: true".to_string()),
        Just("  - Default: 1.0".to_string()),
        Just("  - XML: @attr".to_string()),
    ]
}

/// A whole model, with references confined to objects that exist so the result
/// is usually valid — the interesting failures live just past the valid set.
fn arb_model_text() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(arb_name(), 1..4),
        prop::collection::vec(
            (arb_name(), arb_primitive(), prop::collection::vec(arb_option_line(), 0..3)),
            1..4,
        ),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(object_names, attributes, with_enum, recursive)| {
            let mut text = String::from(
                "---\nprefix: tst\nrepo: \"https://example.org/m/\"\nprefixes:\n  schema: http://schema.org/\n---\n\n",
            );

            for (index, object) in object_names.iter().enumerate() {
                text.push_str(&format!("### {object}{index}\n\n"));
                for (name, dtype, options) in &attributes {
                    text.push_str(&format!("- {name}\n  - Type: {dtype}\n"));
                    for option in options {
                        text.push_str(option);
                        text.push('\n');
                    }
                }
                if recursive {
                    text.push_str(&format!("- self\n  - Type: {object}{index}\n"));
                }
                text.push('\n');
            }

            if with_enum {
                text.push_str("## Enumerations\n\n### Kind\n\n```\nA = \"a\"\nB = \"b\"\n```\n");
            }

            text
        })
}

/// Deliberately malformed documents, assembled from dialect fragments.
fn arb_malformed_text() -> impl Strategy<Value = String> {
    let fragment = prop::sample::select(vec![
        "### O\n",
        "#### O\n",
        "## O\n",
        "# Title\n",
        "- a\n",
        "  - Type: string\n",
        "  - Type: [O](#o)\n",
        "  - Type: Equation\n",
        "  - Term: <x\n",
        "  - Minimum: abc\n",
        "  - MaxLength: *\n",
        "- **b**\n",
        "- *c*\n",
        "- `d`\n",
        "- \n",
        "- ****\n",
        "### A [B]\n",
        "### A [\n",
        "### eq-variable\n",
        "## Enumerations\n",
        "```\nK = \"v\"\n```\n",
        "```py\nA = 1 = 2\n```\n",
        "---\nprefix: t\n---\n",
        "---\nbad: [\n---\n",
        "\t- Type: x\n",
        ": orphan\n",
        "\u{0}\n",
        "é🚀\n",
        "> quote\n",
        "<div>\n",
        "[x](y)\n",
        "\r\n",
        "###\n",
    ]);

    prop::collection::vec(fragment, 1..14).prop_map(|parts| parts.concat())
}

//
// Properties
//

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    /// The parser's contract is `Result<DataModel, Validator>`, so no input
    /// may panic.
    #[test]
    fn prop_parser_never_panics(text in arb_malformed_text()) {
        let _ = DataModel::from_markdown_string(&text);
    }

    /// The same, for documents shaped like real models.
    #[test]
    fn prop_parser_never_panics_on_wellformed_shapes(text in arb_model_text()) {
        let _ = DataModel::from_markdown_string(&text);
    }

    /// A model that validates must export to every target without panicking.
    #[test]
    fn prop_every_target_exports_without_panicking(text in arb_model_text()) {
        let Ok(model) = DataModel::from_markdown_string(&text) else {
            return Ok(());
        };

        for template in ALL_TEMPLATES {
            let mut model = model.clone();
            // JSON Schema needs a root object; the others do not.
            let _ = model.convert_to(template, None);
        }
    }

    /// Every CURIE an RDF target emits must resolve against a declared
    /// prefix, or no RDF tool will load the document.
    #[test]
    fn prop_rdf_prefixes_are_always_declared(text in arb_model_text()) {
        let Ok(model) = DataModel::from_markdown_string(&text) else {
            return Ok(());
        };

        for template in [Templates::Shacl, Templates::Owl, Templates::Shex] {
            let mut model = model.clone();
            let Ok(rendered) = model.convert_to(&template, None) else {
                continue;
            };

            let declared = declared_prefixes(&rendered);
            for prefix in used_prefixes(&rendered) {
                prop_assert!(
                    declared.contains(&prefix),
                    "{template} references undeclared prefix '{prefix}'\ndeclared: {declared:?}\n{rendered}"
                );
            }
        }
    }

    /// An option value is a literal and must survive the parser verbatim.
    #[test]
    fn prop_pattern_values_survive_the_parser(
        pattern in prop::sample::select(vec![
            "^[A-Za-z0-9 _-]+$",
            "^[a-z _]+$",
            "^a _b_ c$",
            "^a *b* c$",
            "^a**b**c$",
            "^a__b__c$",
            "^a  b$",
            "^[A-Z] [a-z]$",
            "^(a|b) c$",
            "^[äöü ß]+$",
        ])
    ) {
        let text = format!(
            "---\nprefix: tst\nrepo: \"https://example.org/m/\"\n---\n\n### O\n\n- a\n  - Type: string\n  - Pattern: {pattern}\n"
        );

        let model = DataModel::from_markdown_string(&text)
            .map_err(|e| TestCaseError::fail(format!("model should parse: {e:?}")))?;

        let stored = model.objects[0].attributes[0]
            .options
            .iter()
            .find(|o| o.key() == "pattern")
            .map(|o| o.value());

        prop_assert_eq!(stored.as_deref(), Some(pattern));
    }

    /// The crate's own round trip must preserve objects and attributes.
    #[test]
    fn prop_markdown_roundtrip_preserves_structure(text in arb_model_text()) {
        let Ok(mut model) = DataModel::from_markdown_string(&text) else {
            return Ok(());
        };

        let Ok(rendered) = model.convert_to(&Templates::Markdown, None) else {
            return Ok(());
        };

        let Ok(reparsed) = DataModel::from_markdown_string(&rendered) else {
            // Re-parsing our own output must not lose the model.
            return Err(TestCaseError::fail(format!(
                "generated Markdown does not parse back:\n{rendered}"
            )));
        };

        let before: Vec<&str> = model.objects.iter().map(|o| o.name.as_str()).collect();
        let after: Vec<&str> = reparsed.objects.iter().map(|o| o.name.as_str()).collect();
        prop_assert_eq!(&before, &after, "objects changed across the round trip:\n{}", rendered);

        for object in &model.objects {
            let restored = reparsed
                .objects
                .iter()
                .find(|o| o.name == object.name)
                .expect("object present");
            let before: Vec<&str> = object.attributes.iter().map(|a| a.name.as_str()).collect();
            let after: Vec<&str> = restored.attributes.iter().map(|a| a.name.as_str()).collect();
            prop_assert_eq!(&before, &after, "attributes of {} changed", object.name);
        }
    }
}

//
// Helpers
//

/// Prefix names declared by `@prefix x: <...>` / `PREFIX x: <...>`.
fn declared_prefixes(rendered: &str) -> HashSet<String> {
    rendered
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line
                .strip_prefix("@prefix ")
                .or_else(|| line.strip_prefix("PREFIX "))?;
            rest.split(':').next().map(|s| s.trim().to_string())
        })
        .collect()
}

/// Prefixes referenced by a CURIE, ignoring declarations, comments, absolute
/// IRIs and the contents of string literals.
fn used_prefixes(rendered: &str) -> HashSet<String> {
    let mut used = HashSet::new();

    for line in rendered.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("@prefix")
            || trimmed.starts_with("PREFIX")
            || trimmed.starts_with('#')
        {
            continue;
        }

        // Drop string literals so URIs inside quotes are not read as CURIEs.
        let mut outside = String::new();
        let mut in_string = false;
        for character in trimmed.chars() {
            if character == '"' {
                in_string = !in_string;
                outside.push(' ');
            } else if !in_string {
                outside.push(character);
            }
        }

        for token in outside.split_whitespace() {
            let token = token
                .trim_matches(|c: char| !c.is_alphanumeric() && c != ':' && c != '_' && c != '-');
            if token.starts_with('<') || token.contains("://") || !token.contains(':') {
                continue;
            }
            if let Some(prefix) = token.split(':').next() {
                if !prefix.is_empty()
                    && prefix
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    used.insert(prefix.to_string());
                }
            }
        }
    }

    used
}

//
// Regression corpus: inputs kept as files so each case survives independently
// of the generator that produced it.
//

#[test]
fn test_fuzz_corpus_never_panics() {
    let corpus = std::path::Path::new("tests/data/fuzz");
    if !corpus.is_dir() {
        return;
    }

    let mut checked = 0;
    for entry in std::fs::read_dir(corpus).expect("could not read corpus directory") {
        let path = entry.expect("corpus entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = std::fs::read_to_string(&path).expect("could not read corpus file");
        // Must return, whatever the verdict.
        let _ = DataModel::from_markdown_string(&content);
        checked += 1;
    }

    assert!(checked > 0, "the fuzz corpus is empty");
}
