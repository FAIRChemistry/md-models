extern crate mdmodels;

#[cfg(test)]
mod tests {
    use mdmodels::{self, markdown::parser::parse_markdown};
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[test]
    fn test_parse() {
        // Arrange
        let path = Path::new("tests/data/model.md");

        // Act
        let model = parse_markdown(path).expect("Could not parse markdown");

        // Assert
        // Check if there are two objects
        assert_eq!(model.objects.len(), 2);

        // Check if objects Test1 and Test2 are present
        let obj_names: Vec<String> = model.objects.iter().map(|o| o.name.clone()).collect();
        assert!(obj_names.contains(&"Test".to_string()));
        assert!(obj_names.contains(&"Test2".to_string()));

        // Check if the attributes are present
        let test1 = model.objects.iter().find(|o| o.name == *"Test").unwrap();

        let test2 = model.objects.iter().find(|o| o.name == *"Test2").unwrap();

        assert_eq!(test1.attributes.len(), 4);
        assert_eq!(test2.attributes.len(), 2);

        // Check if the attributes are correct
        let test1_attr_names: Vec<String> =
            test1.attributes.iter().map(|a| a.name.clone()).collect();
        let expected = vec![
            "name".to_string(),
            "number".to_string(),
            "test2".to_string(),
            "ontology".to_string(),
        ];

        assert_eq!(test1_attr_names, expected);

        // Check if the datatypes are correct
        let test1_name_attribute = test1.attributes.iter().find(|a| a.name == *"name").unwrap();

        assert!(test1_name_attribute.dtypes.contains(&"string".to_string()));

        // Check if multiple datatypes are correct
        let test2_names_attribute = test2
            .attributes
            .iter()
            .find(|a| a.name == *"names")
            .unwrap();

        assert!(test2_names_attribute.dtypes.contains(&"string".to_string()));
        assert!(test2_names_attribute.is_array);
    }

    #[test]
    fn test_json_schema() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = parse_markdown(path).expect("Could not parse markdown");

        // Act
        let schema = model.json_schema("Test".to_string());
        let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();

        // Assert
        let expected_schema =
            std::fs::read_to_string("tests/data/expected_json_schema.json").unwrap();
        // Parse with serde_json
        let expected_schema: serde_json::Value = serde_json::from_str(&expected_schema).unwrap();

        assert_eq!(schema, expected_schema);
    }

    #[test]
    #[should_panic]
    fn test_json_schema_no_objects() {
        // Arrange
        let model = mdmodels::datamodel::DataModel::new(None, None);

        // Act
        model.json_schema("Test".to_string());
    }

    #[test]
    #[should_panic]
    fn test_json_schema_no_object() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = parse_markdown(path).expect("Could not parse markdown");

        // Act
        model.json_schema("Test3".to_string());
    }

    #[test]
    fn test_sdrdm_schema() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = parse_markdown(path).expect("Could not parse markdown");

        // Act
        let schema = model.sdrdm_schema();
        let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();

        // Assert
        let expected_schema =
            std::fs::read_to_string("tests/data/expected_sdrdm_schema.json").unwrap();
        let expected_schema: serde_json::Value = serde_json::from_str(&expected_schema).unwrap();

        assert_eq!(schema, expected_schema);
    }

    #[test]
    #[should_panic]
    fn test_sdrdm_schema_no_objects() {
        // Arrange
        let model = mdmodels::datamodel::DataModel::new(None, None);

        // Act
        model.sdrdm_schema();
    }

    #[test]
    fn test_json_schema_all() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = parse_markdown(path).expect("Could not parse markdown");

        // Act
        model.json_schema_all("tests/intermediates/".to_string());

        // Assert
        let filenames = vec!["Test.json", "Test2.json"];
        for filename in filenames {
            let obj_name = filename.replace(".json", "");
            let expected_schema =
                std::fs::read_to_string(format!("tests/intermediates/{}", filename)).unwrap();
            let schema = model.json_schema(obj_name);
            assert_eq!(schema.trim(), expected_schema.trim());
        }
    }
}
