use std::path::Path;

use mdmodels::datamodel::DataModel;

fn main() {
    let path = Path::new("tests/data/model_inheritance.md");

    let model = DataModel::from_markdown(path).expect("Could not parse markdown");

    let schema = model.sdrdm_schema();

    println!("{}", schema);
}
