//! This module provides the main entry point for the application and includes tests
//! for the generated data structures.
//!
//! The tests demonstrate:
//! - Direct struct creation and field access
//! - Builder pattern usage for constructing complex objects
//! - Handling of optional fields and enum variants
//! - JSON-LD header creation and manipulation
//! - Default JSON-LD header generation

mod generated;

#[cfg(test)]
mod tests {
    use crate::generated::{
        JsonLdContext, JsonLdHeader, SimpleContext, TermDef, Test2Builder, TestBuilder, TypeOrVec,
        default_test_jsonld_header,
    };

    use super::*;
    use generated::{Ontology, Test, Test2, TestNumberType};

    #[test]
    fn test_struct_creation() {
        let test = Test {
            name: "test1".to_string(),
            number: Some(TestNumberType::F64(42.0)),
            test2: vec![Test2 {
                names: vec!["name1".to_string(), "name2".to_string()],
                number: Some(123.45),
                ..Default::default()
            }],
            ontology: Some(Ontology::Go),
            ..Default::default()
        };

        assert_eq!(test.name, "test1");
        if let Some(TestNumberType::F64(num)) = test.number {
            assert_eq!(num, 42.0);
        } else {
            panic!("Expected F64 number type");
        }

        let test2 = test.test2;
        assert_eq!(test2[0].names[0], "name1");
        assert_eq!(test2[0].number, Some(123.45));
    }

    #[test]
    fn test_builder() {
        let test = TestBuilder::default()
            .test2(vec![
                Test2Builder::default()
                    .names(vec!["name1".to_string(), "name2".to_string()])
                    .number(Some(123.45))
                    .build()
                    .unwrap(),
            ])
            .name("test2".to_string())
            .number(Some(TestNumberType::String("42".to_string())))
            .ontology(Some(Ontology::Go))
            .build()
            .unwrap();

        assert_eq!(test.name, "test2");
        if let Some(TestNumberType::String(s)) = test.number {
            assert_eq!(s, "42");
        } else {
            panic!("Expected String number type");
        }
    }

    #[test]
    fn test_jsonld_header() {
        let mut header = JsonLdHeader::default();
        header.add_term(
            "name",
            TermDef::Simple("https://schema.org/name".to_string()),
        );
        header.update_term(
            "name",
            TermDef::Simple("https://example.org/fullName".to_string()),
        );
        header.remove_term("name");
    }

    #[test]
    fn test_default_jsonld_header() {
        let header = JsonLdHeader::default();
        assert_eq!(header.context, None);
        assert_eq!(header.id, None);
        assert_eq!(header.type_, None);
    }

    #[test]
    fn test_default_in_generated() {
        let test = Test::default();
        assert_eq!(test.name, "2.0");
        assert!(test.test2.is_empty());
        assert!(test.ontology.is_none());
        assert!(test.number.is_none());

        // Test the default JSON-LD header for Test struct
        let test_header = default_test_jsonld_header();
        assert!(test_header.is_some());

        let header = test_header.unwrap();
        assert!(header.context.is_some());
        assert!(header.id.is_some());
        assert!(header.type_.is_some());

        // Verify the ID format
        if let Some(id) = &header.id {
            assert!(id.starts_with("tst:Test/"));
        }

        // Verify the type
        if let Some(TypeOrVec::Multi(types)) = &header.type_ {
            assert_eq!(types, &vec!["tst:Test".to_string()]);
        }

        // Test the context contains expected terms
        if let Some(JsonLdContext::Object(context)) = &header.context {
            assert!(context.terms.contains_key("tst"));
            assert!(context.terms.contains_key("schema"));
            assert!(context.terms.contains_key("name"));
            assert!(context.terms.contains_key("number"));
            assert!(context.terms.contains_key("test2"));
        }
    }
}
