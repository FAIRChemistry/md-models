//! This module provides the main entry point for the application and includes tests
//! for the generated data structures.
//!
//! The tests demonstrate:
//! - Direct struct creation and field access
//! - Builder pattern usage for constructing complex objects
//! - Handling of optional fields and enum variants

mod generated;

#[cfg(test)]
mod tests {
    use crate::generated::{Test2Builder, TestBuilder};

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
            }],
            ontology: Some(Ontology::Go),
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
            .test2(vec![Test2Builder::default()
                .names(vec!["name1".to_string(), "name2".to_string()])
                .number(Some(123.45))
                .build()
                .unwrap()])
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
}
