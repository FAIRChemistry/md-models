# MD-Models Types

This page contains types that the MD-Models library provides from of the shelf. You can use these types in your own data models by simply referencing them in your data model's attributes `Type` section.

## Types available

- [Unit Definition](./unit-definition.md)
- [Equation](./equation.md)

## How to use

To use a type in your data model, simply reference it in the `Type` section of your data model's attribute. For example, to use the `UnitDefinition` type in your data model, you would write:

```markdown
- unit
  - Type: UnitDefinition
  - Description: Unit of the estimated parameter.
```

This will add the `UnitDefinition` type to your data model and can thus be used across your data model. If you like to modify these types, feel free to copy the markdown file to your own project and modify it as you see fit.


## Contributing

If you have a type that you think would be useful for others, feel free to create a pull request to add it to this repository. We are happy to accept any contributions that you think would be useful for others.

If you want to propose changes to an existing type, please only edit the corresponding markdown file in this repository. The continuous integration will automatically update the repository in your branch with the changes you propose. After the CI has successfully run, you can create a pull request to merge your changes into the main branch.
