# Schema validation

MD-Models provides comprehensive validation capabilities to ensure your data models are well-formed, consistent, and ready for code generation. The validation system checks for structural integrity, naming conventions, type consistency, and other potential issues that could cause problems during code generation or runtime.

## The Validate Command

The `validate` command checks your MD-Models file for errors and inconsistencies. It can validate both markdown model files and JSON Schema files.

### Basic Usage

```bash
md-models validate -i <input>
```

**Examples:**

```bash
# Validate a local markdown model file
md-models validate -i model.md

# Validate a model from a remote URL
md-models validate -i https://example.com/model.md

# Validate a JSON Schema file
md-models validate -i schema.json
```

### Input Sources

The validate command accepts the same input types as the convert command:

- **Local file path**: Path to a markdown model or JSON Schema file on your local filesystem
- **Remote URL**: URL to a markdown model or JSON Schema file hosted online

MD-Models automatically detects the file type (markdown model vs JSON Schema) and applies the appropriate validation rules.

## Validation Checks

MD-Models performs comprehensive validation checks on your data model. Understanding these checks helps you write correct models and quickly identify issues.

### Global Model Validation

#### Empty Model Check

**What it checks:** Ensures your model contains at least one object definition.

**Error message:** `"This model has no definitions."`

**Solution:** Add at least one object to your model.

**Example:**

```markdown
# My Model

<!-- This would fail validation - no objects defined -->
```

#### Duplicate Object Names

**What it checks:** Ensures each object has a unique name within the model.

**Error type:** `DuplicateError`

**Error message:** `"Object '<name>' is defined more than once."`

**Solution:** Rename one of the duplicate objects to be unique.

**Example:**

```markdown
## User
- name: string

## User  <!-- Error: duplicate name -->
- email: string
```

#### Duplicate Enum Names

**What it checks:** Ensures each enumeration has a unique name within the model.

**Error type:** `DuplicateError`

**Error message:** `"Enumeration '<name>' is defined more than once."`

**Solution:** Rename one of the duplicate enumerations to be unique.

### Object Validation

#### Empty Object Check

**What it checks:** Ensures objects have at least one attribute (unless `allow_empty: true` is set in frontmatter).

**Error type:** `ObjectError`

**Error message:** `"Type '<name>' is empty and has no properties."`

**Solution:** Add at least one property to the object, or set `allow_empty: true` in the model's frontmatter if empty objects are intentional.

**Example:**

```markdown
## EmptyObject
<!-- This would fail validation - no attributes -->
```

#### Object Name Validation

**What it checks:** Validates that object names follow naming conventions:

1. **Must start with a letter**: Object names cannot start with numbers or special characters
2. **No whitespace**: Object names cannot contain spaces
3. **No special characters**: Only alphanumeric characters and underscores are allowed

**Error type:** `NameError`

**Error messages:**

- `"Name '<name>' must start with a letter."`
- `"Name '<name>' contains whitespace, which is not valid. Use underscores instead."`
- `"Name '<name>' contains special characters, which are not valid except for underscores."`

**Valid examples:**

- `User`
- `UserProfile`
- `user_profile`
- `User123`

**Invalid examples:**

- `123User` (starts with number)
- `User Profile` (contains space)
- `User-Profile` (contains special character)
- `User.Profile` (contains special character)

### Attribute Validation

#### Duplicate Attribute Names

**What it checks:** Ensures each attribute within an object has a unique name.

**Error type:** `DuplicateError`

**Error message:** `"Property '<name>' is defined more than once."`

**Solution:** Rename one of the duplicate properties to be unique within the object.

**Example:**

```markdown
## User
- name: string
- name: string  <!-- Error: duplicate attribute name -->
```

#### Attribute Name Validation

**What it checks:** Validates that attribute names follow the same naming conventions as object names:

1. **Must start with a letter**
2. **No whitespace**
3. **No special characters** (except underscores)

**Error type:** `NameError`

**Error messages:** Same as object name validation

**Valid examples:**

- `name`
- `user_name`
- `emailAddress`
- `age123`

**Invalid examples:**

- `123age` (starts with number)
- `user name` (contains space)
- `user-name` (contains special character)

#### Type Definition Validation

**What it checks:** Ensures every attribute has at least one valid type defined.

**Error type:** `TypeError`

**Error messages:**

- `"Property '<name>' has no type specified."`
- `"Property '<name>' has no type defined. Either define a type or use a base type."`

**Solution:** Add a type to the property using the syntax `- <property>: <TYPE>`.

**Example:**

```markdown
## User
- name  <!-- Error: no type specified -->
- email: string  <!-- Valid -->
```

#### Type Reference Validation

**What it checks:** Ensures that all referenced types exist in the model or are basic types.

**Error type:** `TypeError`

**Error message:** `"Type '<type>' of property '<name>' not found."`

**Solution:** Either add the referenced type to your model, or use one of the basic types: `string`, `number`, `integer`, `boolean`, `float`, `date`, `bytes`.

**Valid basic types:**

- `string` - Text data
- `number` - Numeric value
- `integer` - Whole number
- `boolean` - True/false value
- `float` - Floating-point number
- `date` - Date value
- `bytes` - Binary data

**Example:**

```markdown
## User
- name: string  <!-- Valid: basic type -->
- profile: UserProfile  <!-- Error if UserProfile doesn't exist -->
- status: Status  <!-- Valid if Status enum exists -->
```

### XML Option Validation

MD-Models validates XML serialization options to ensure they're correctly formatted.

#### XML Element Option Validation

**What it checks:** Validates XML element options (used for custom XML tag names).

**Error type:** `XMLError`

**Error messages:**

- `"XML option is not defined."`
- `"Name '<name>' contains special characters..."`

**Solution:** Ensure XML options are properly defined and don't contain invalid characters.

#### XML Attribute Option Validation

**What it checks:** Validates XML attribute options (used for XML attributes).

**Error type:** `XMLError`

**Error messages:**

- `"XML attribute option is not defined."`
- `"Name '<name>' contains special characters..."`

**Solution:** Ensure XML attribute options use the `@` prefix and are properly formatted.

#### XML Wrapped Option Validation

**What it checks:** Validates XML wrapped element options (for nested XML structures).

**Error type:** `XMLError`

**Error messages:**

- `"XML wrapped option can only contain two types."`
- `"Name '<name>' contains special characters..."`

**Solution:** XML wrapped options can only have a depth of two types. For deeper nesting, create intermediate objects.

## Error Reporting

When validation fails, MD-Models provides detailed error messages to help you identify and fix issues.

### Error Format

Each validation error includes:

- **Line number**: The line(s) where the error occurs
- **Location**: The object and attribute (if applicable) where the error was found
- **Error type**: The category of error (NameError, TypeError, DuplicateError, etc.)
- **Message**: A clear description of what's wrong
- **Solution**: Suggested fix for the error

### Example Error Output

```
[line: 5, 12] [User.name] NameError:
 └── Name 'user name' contains whitespace, which is not valid. Use underscores instead.
     Resolve the issue by using 'user_name'.

[line: 8] [User] TypeError:
 └── Type 'Profile' of property 'profile' not found.
     Add the type 'Profile' to the model or use a base type.
```

### Error Types

MD-Models categorizes errors into several types:

- **`NameError`**: Issues with naming conventions (object names, attribute names)
- **`TypeError`**: Issues with type definitions or type references
- **`DuplicateError`**: Duplicate definitions (objects, enums, attributes)
- **`ObjectError`**: Issues with object structure (empty objects)
- **`XMLError`**: Issues with XML serialization options
- **`GlobalError`**: Model-level issues (empty model)

## Validation Success

When validation passes, you'll see:

```
✓ Model is valid
```

This indicates your model is well-formed and ready for code generation or export.

## JSON Schema Validation

MD-Models can also validate JSON Schema files. When you provide a JSON Schema file as input, MD-Models:

1. Parses the JSON Schema
2. Converts it to an internal model representation
3. Validates the converted model using the same validation rules

This allows you to validate JSON Schema files and ensure they're compatible with MD-Models workflows.

**Example:**

```bash
md-models validate -i schema.json
```

## Best Practices

1. **Validate before generating**: Always validate your model before running code generation to catch errors early
2. **Fix errors systematically**: Start with global errors (duplicates, empty model) before fixing attribute-level errors
3. **Use descriptive names**: Follow naming conventions to avoid NameError issues
4. **Check type references**: Ensure all referenced types exist in your model or are basic types
5. **Validate in CI/CD**: Include validation checks in your continuous integration pipeline

## Integration with Code Generation

Validation is automatically performed when parsing models for code generation. If validation fails during parsing, code generation will not proceed, ensuring that only valid models are used to generate code.

You can also use validation independently to check models without generating code:

```bash
# Just validate
md-models validate -i model.md

# Validate and generate (validation happens automatically)
md-models convert -i model.md -t python-pydantic -o models.py
```
