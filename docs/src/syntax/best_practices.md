# Best practices

1. **Use Descriptive Names**
   - Object names should be PascalCase (e.g., `ResearchPublication`)
   - Attribute names should be in snake_case (e.g., `publication_year`)
   - Use clear, domain-specific terminology

2. **Identifiers**
   - Mark primary keys with double underscores (e.g., `__doi__`)
   - Choose meaningful identifier fields

3. **Documentation**
   - Always include object descriptions
   - Document complex attributes
   - Explain any constraints or business rules

4. **Semantic Mapping**
   - Use standard vocabularies when possible
   - Define custom terms in your prefix map
   - Maintain consistent terminology

5. **Validation Rules**
   - Include range constraints for numbers
   - Specify default values when appropriate
   - Document any special validation requirements

## Common Patterns

### Array Types

```markdown
- tags
  - Type: string[]
  - Description: List of keywords describing the publication
```

### Object References

```markdown
- main_author
  - Type: Author
  - Description: The primary author of the publication
```

### Required Fields

```markdown
- __id__
  - Type: Identifier
  - Description: Unique identifier for the object
```

Remember that MD-Models aims to balance human readability with technical precision. Your object definitions should be clear enough for domain experts to understand while maintaining the structure needed for technical implementation.