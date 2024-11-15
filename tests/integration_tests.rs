/*
 * Copyright (c) 2024 Jan Range
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */

extern crate mdmodels;

#[cfg(test)]
mod tests {
    use mdmodels::{self, datamodel::DataModel};
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[test]
    fn test_parse() {
        // Arrange
        let path = Path::new("tests/data/model.md");

        // Act
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

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
    fn test_full_model() {
        // Arrange
        let path = Path::new("tests/data/model_full_documentation.md");

        // Act
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Assert
        let expected =
            std::fs::read_to_string("tests/data/expected_sdrdm_full_schema.json").unwrap();
        let expected: serde_json::Value = serde_json::from_str(&expected).unwrap();

        let schema = model.sdrdm_schema();
        let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(schema, expected);
    }

    #[test]
    #[should_panic]
    fn test_parse_no_objects() {
        // Arrange
        let path = Path::new("tests/data/model_no_objects.md");

        // Act
        DataModel::from_markdown(path).expect("Could not parse markdown");
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid() {
        // Arrange
        let path = Path::new("tests/data/model_missing_types.md");

        // Act
        DataModel::from_markdown(path).expect("Could not parse markdown");
    }

    #[test]
    #[should_panic]
    fn test_duplicate_objects() {
        // Arrange
        let path = Path::new("tests/data/model_duplicates.md");

        // Act
        DataModel::from_markdown(path).expect("Could not parse markdown");
    }

    #[test]
    fn test_json_schema_known_obj() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Act
        let schema = model.json_schema(Some("Test".to_string()));
        let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();

        // Assert
        let expected_schema =
            std::fs::read_to_string("tests/data/expected_json_schema.json").unwrap();
        // Parse with serde_json
        let expected_schema: serde_json::Value = serde_json::from_str(&expected_schema).unwrap();

        assert_eq!(schema, expected_schema);
    }

    #[test]
    fn test_json_schema_unknown_obj() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Act
        let schema = model.json_schema(None);
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
        model.json_schema(Some("Test".to_string()));
    }

    #[test]
    #[should_panic]
    fn test_json_schema_no_object() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Act
        model.json_schema(Some("Test3".to_string()));
    }

    #[test]
    fn test_sdrdm_schema() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

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
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Act
        model.json_schema_all("tests/intermediates/".to_string());

        // Assert
        let filenames = vec!["Test.json", "Test2.json"];
        for filename in filenames {
            let obj_name = filename.replace(".json", "");
            let expected_schema =
                std::fs::read_to_string(format!("tests/intermediates/{}", filename)).unwrap();
            let schema = model.json_schema(Some(obj_name));

            assert_eq!(
                serde_json::from_str::<serde_json::Value>(schema.as_str())
                    .expect("Could not parse generated schema"),
                serde_json::from_str::<serde_json::Value>(expected_schema.as_str())
                    .expect("Could not parse expected schema")
            );
        }
    }

    #[test]
    fn test_model_merge() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let mut model = DataModel::from_markdown(path).expect("Could not parse markdown");
        let path2 = Path::new("tests/data/model_merge.md");
        let model2 = DataModel::from_markdown(path2).expect("Could not parse markdown");

        // Act
        model.merge(&model2);

        // Assert
        let obj_names: Vec<String> = model.objects.iter().map(|o| o.name.clone()).collect();

        assert_eq!(model.objects.len(), 3);
        assert!(obj_names.contains(&"Test".to_string()));
        assert!(obj_names.contains(&"Test2".to_string()));
        assert!(obj_names.contains(&"Added".to_string()));

        let enum_names: Vec<String> = model.enums.iter().map(|e| e.name.clone()).collect();

        assert_eq!(model.enums.len(), 2);
        assert!(enum_names.contains(&"Ontology".to_string()));
        assert!(enum_names.contains(&"AddedEnum".to_string()));
    }

    #[test]
    #[should_panic]
    fn test_model_merge_invalid() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let mut model = DataModel::from_markdown(path).expect("Could not parse markdown");
        let path2 = Path::new("tests/data/model_merge_invalid.md");
        let model2 = DataModel::from_markdown(path2).expect("Could not parse markdown");

        // Act
        model.merge(&model2);
    }

    #[test]
    #[should_panic]
    fn test_inheritance_invalid() {
        // Arrange
        let path = Path::new("tests/data/model_inheritance_invalid.md");

        // Act
        DataModel::from_markdown(path).expect("Could not parse markdown");
    }

    #[test]
    #[should_panic]
    fn test_inheritance() {
        // Arrange
        let path = Path::new("tests/data/model_inheritance_invalid.md");

        // Act
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Assert
        let schema = model.sdrdm_schema();
        let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();

        let expected_schema =
            std::fs::read_to_string("tests/data/expected_sdrdm_schema_inheritance.json").unwrap();

        assert_eq!(schema, expected_schema);
    }

    #[test]
    fn test_no_frontmatter() {
        // Arrange
        let path = Path::new("tests/data/model_no_frontmatter.md");

        // Act
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Assert
        assert_eq!(model.objects.len(), 1);
        assert_eq!(model.enums.len(), 1);
    }

    #[test]
    fn test_multiple_keyword() {
        // Arrange
        let path = Path::new("tests/data/model_multiple_keyword.md");

        // Act
        let model = DataModel::from_markdown(path).expect("Could not parse markdown");

        // Assert
        assert_eq!(model.objects.len(), 1);
        assert_eq!(model.objects[0].attributes.len(), 1);
        assert!(model.objects[0].attributes[0].is_array);
    }

    #[test]
    #[should_panic]
    fn test_invalid_names() {
        // Arrange
        let path = Path::new("tests/data/model_invalid_names.md");

        // Act
        DataModel::from_markdown(path).expect("Could not parse markdown");
    }
}
