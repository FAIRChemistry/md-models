extern crate sdrdm;

#[cfg(test)]
mod tests {
    use sdrdm;

    #[test]
    fn test_parse() {
        // Arrange
        let path = "tests/data/model.md".to_string();

        // Act
        let model = sdrdm::DataModel::parse(path);

        // Assert
        // Check if there are two objects
        assert_eq!(model.objects.len(), 2);

        // Check if objects Test1 and Test2 are present
        let obj_names: Vec<String> = model.objects.iter().map(|o| o.name.clone()).collect();
        assert!(obj_names.contains(&"Test".to_string()));
        assert!(obj_names.contains(&"Test2".to_string()));

        // Check if the attributes are present
        let test1 = model
            .objects
            .iter()
            .find(|o| o.name == "Test".to_string())
            .unwrap();

        let test2 = model
            .objects
            .iter()
            .find(|o| o.name == "Test2".to_string())
            .unwrap();

        assert_eq!(test1.attributes.len(), 3);
        assert_eq!(test2.attributes.len(), 2);

        // Check if the attributes are correct
        let test1_attr_names: Vec<String> =
            test1.attributes.iter().map(|a| a.name.clone()).collect();
        let expected = vec![
            "name".to_string(),
            "number".to_string(),
            "test2".to_string(),
        ];

        assert_eq!(test1_attr_names, expected);

        // Check if the datatypes are correct
        let test1_name_attribute = test1
            .attributes
            .iter()
            .find(|a| a.name == "name".to_string())
            .unwrap();

        assert!(test1_name_attribute.dtypes.contains(&"string".to_string()));

        // Check if multiple datatypes are correct
        let test2_names_attribute = test2
            .attributes
            .iter()
            .find(|a| a.name == "names".to_string())
            .unwrap();

        assert!(test2_names_attribute.dtypes.contains(&"string".to_string()));
        assert!(test2_names_attribute.is_array == true);
    }

    #[test]
    fn test_json_schema() {
        // Arrange
        let path = "tests/data/model.md".to_string();
        let model = sdrdm::DataModel::parse(path);

        // Act
        let schema = model.json_schema("Test".to_string());

        // Assert
        let expected_schema =
            std::fs::read_to_string("tests/data/expected_json_schema.json").unwrap();

        assert_eq!(schema.trim(), expected_schema.trim());
    }

    #[test]
    #[should_panic]
    fn test_json_schema_no_objects() {
        // Arrange
        let model = sdrdm::DataModel::new();

        // Act
        model.json_schema("Test".to_string());
    }

    #[test]
    #[should_panic]
    fn test_json_schema_no_object() {
        // Arrange
        let path = "tests/data/model.md".to_string();
        let model = sdrdm::DataModel::parse(path);

        // Act
        model.json_schema("Test3".to_string());
    }

    #[test]
    fn test_sdrdm_schema() {
        // Arrange
        let path = "tests/data/model.md".to_string();
        let model = sdrdm::DataModel::parse(path);

        // Act
        let schema = model.sdrdm_schema();

        // Assert
        let expected_schema =
            std::fs::read_to_string("tests/data/expected_sdrdm_schema.json").unwrap();

        assert_eq!(schema.trim(), expected_schema.trim());
    }

    #[test]
    #[should_panic]
    fn test_sdrdm_schema_no_objects() {
        // Arrange
        let model = sdrdm::DataModel::new();

        // Act
        model.sdrdm_schema();
    }

    #[test]
    fn test_json_schema_all() {
        // Arrange
        let model = sdrdm::DataModel::parse("tests/data/model.md".to_string());

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
