# My model is not validating

This section highlights the most common mistakes that break validation rules. Understanding these pitfalls will help you write valid models and avoid errors during validation and code generation.

## Naming Issues

### ❌ Names Starting with Numbers

**Problem:** Object and attribute names cannot start with a number.

```markdown
### 1Test          <!-- ❌ Invalid: starts with number -->
- 1number: string <!-- ❌ Invalid: starts with number -->
```

**Solution:** Use letters at the beginning of names.

```markdown
### Test1          <!-- ✅ Valid -->
- number1: string <!-- ✅ Valid -->
```

### ❌ Names with Whitespace

**Problem:** Object and attribute names cannot contain spaces.

```markdown
### Test Object    <!-- ❌ Invalid: contains space -->
- some name: string <!-- ❌ Invalid: contains space -->
```

**Solution:** Use underscores or camelCase instead.

```markdown
### TestObject     <!-- ✅ Valid -->
- some_name: string <!-- ✅ Valid -->
- someName: string <!-- ✅ Valid -->
```

### ❌ Names with Special Characters

**Problem:** Only alphanumeric characters and underscores are allowed in names.

```markdown
### User-Profile   <!-- ❌ Invalid: contains hyphen -->
- user.name: string <!-- ❌ Invalid: contains dot -->
```

**Solution:** Use underscores or camelCase.

```markdown
### UserProfile    <!-- ✅ Valid -->
- user_name: string <!-- ✅ Valid -->
```

### Type Definition Issues

#### ❌ Missing Type Definitions

**Problem:** Every attribute must have a type specified.

```markdown
### User
- name            <!-- ❌ Invalid: no type -->
- email: string   <!-- ✅ Valid -->
```

**Solution:** Always specify a type using `- <property>: <TYPE>` syntax.

```markdown
### User
- name: string    <!-- ✅ Valid -->
- email: string   <!-- ✅ Valid -->
```

#### ❌ Undefined Type References

**Problem:** Referenced types must exist in the model or be basic types.

```markdown
### User
- profile: UserProfile <!-- ❌ Invalid if UserProfile doesn't exist -->
- name: string          <!-- ✅ Valid: basic type -->
```

**Solution:** Either define the referenced type or use a basic type (`string`, `number`, `integer`, `boolean`, `float`, `date`, `bytes`).

```markdown
### UserProfile    <!-- Define the type first -->
- bio: string

### User
- profile: UserProfile <!-- ✅ Valid -->
```

#### ❌ Incorrect Type Syntax

**Problem:** Using wrong keywords or syntax for type definitions.

```markdown
### User
- name
  - DType: string <!-- ❌ Invalid: should be "Type" not "DType" -->
```

**Solution:** Use the correct syntax: `- <property>: <TYPE>` or `- Type: <TYPE>`.

```markdown
### User
- name: string    <!-- ✅ Valid: inline syntax -->
- email            <!-- ✅ Valid: block syntax -->
  - Type: string
```

### Duplicate Definitions

#### ❌ Duplicate Object Names

**Problem:** Each object must have a unique name within the model.

```markdown
### User
- name: string

### User  <!-- ❌ Invalid: duplicate name -->
- email: string
```

**Solution:** Rename one of the duplicate objects.

```markdown
### User
- name: string

### UserProfile  <!-- ✅ Valid: unique name -->
- email: string
```

#### ❌ Duplicate Attribute Names

**Problem:** Each attribute within an object must have a unique name.

```markdown
### User
- name: string
- name: string  <!-- ❌ Invalid: duplicate attribute -->
```

**Solution:** Rename one of the duplicate attributes or combine them if they represent the same concept.

```markdown
### User
- first_name: string
- last_name: string  <!-- ✅ Valid: unique names -->
```

#### ❌ Duplicate Enum Names

**Problem:** Each enumeration must have a unique name within the model.

````markdown
### Status

```text
ACTIVE = "active"
```

### Status  <!-- ❌ Invalid: duplicate enum name -->

```text
INACTIVE = "inactive"
```
````

**Solution:** Rename one of the duplicate enumerations.

````markdown
### UserStatus

```text
ACTIVE = "active"
```

### AccountStatus  <!-- ✅ Valid: unique name -->

```text
INACTIVE = "inactive"
```
````

### Structure Issues

#### ❌ Empty Models

**Problem:** Models must contain at least one object definition.

```markdown
# My Model

<!-- ❌ Invalid: no objects defined -->
```

**Solution:** Add at least one object to your model.

```markdown
# My Model

### User
- name: string  <!-- ✅ Valid: model has objects -->
```

#### ❌ Empty Objects

**Problem:** Objects must have at least one attribute (unless `allow_empty: true` is set).

```markdown
### EmptyObject
<!-- ❌ Invalid: no attributes -->
```

**Solution:** Add at least one property or set `allow_empty: true` in frontmatter.

```markdown
---
allow_empty: true
---

### EmptyObject  <!-- ✅ Valid with allow_empty -->
```

Or:

```markdown
### EmptyObject
- id: string  <!-- ✅ Valid: has at least one attribute -->
```

### XML Option Issues

#### ❌ Empty XML Options

**Problem:** XML options cannot be empty or contain only special characters.

```markdown
### User
- name: string
  - XML:        <!-- ❌ Invalid: empty -->
  - XML: @      <!-- ❌ Invalid: only special character -->
```

**Solution:** Provide a valid XML element name or use proper attribute syntax.

```markdown
### User
- name: string
  - XML: userName  <!-- ✅ Valid: element name -->
  - XML: @id       <!-- ✅ Valid: attribute with name -->
```

#### ❌ Special Characters in XML Options

**Problem:** XML element and attribute names cannot contain special characters like colons (unless part of a namespace prefix).

```markdown
### User
- name: string
  - XML: schema:hello  <!-- ❌ Invalid: colon not allowed in element names -->
  - XML: @schema:hello <!-- ❌ Invalid: colon not allowed in attribute names -->
```

**Solution:** Use valid XML names without special characters, or restructure your XML serialization.

```markdown
### User
- name: string
  - XML: hello         <!-- ✅ Valid: simple name -->
  - XML: @hello        <!-- ✅ Valid: attribute -->
```

#### ❌ Invalid XML Wrapped Syntax

**Problem:** XML wrapped options can only contain two levels of nesting.

```markdown
### Test
- items: string[]
  - XML: some/other/path  <!-- ❌ Invalid: more than 2 levels -->
```

**Solution:** Limit XML wrapped paths to two levels, or create intermediate objects for deeper nesting.

```markdown
### Test
- items: string[]
  - XML: items/item  <!-- ✅ Valid: exactly 2 levels -->
```

Or create intermediate objects:

```markdown
### ItemList
- items: Item[]

### Item
- value: string

### Test
- list: ItemList
  - XML: list/items  <!-- ✅ Valid: uses intermediate object -->
```

#### ❌ Invalid Multiple Types with XML

**Problem:** When using multiple types, XML options must match the number of types.

```markdown
### Test
- value: string, float
  - XML: fine, bad:  <!-- ❌ Invalid: second XML option has colon -->
```

**Solution:** Provide valid XML options for each type, matching the type count.

```markdown
### Test
- value: string, float
  - XML: stringValue, floatValue  <!-- ✅ Valid: matches type count -->
```

### Reserved Names

#### ❌ Using Reserved Names

**Problem:** Certain names like `__other__` are reserved and cannot be used as attribute names.

```markdown
### Test
- __other__: string  <!-- ❌ Invalid: reserved name -->
```

**Solution:** Use a different name that doesn't conflict with reserved keywords.

```markdown
### Test
- other_value: string  <!-- ✅ Valid: not reserved -->
```

### Inheritance Issues

#### ❌ Invalid Inheritance Syntax

**Problem:** Inheritance syntax must be properly formatted.

```markdown
### Test [  <!-- ❌ Invalid: incomplete syntax -->
- name: string
```

**Solution:** Use proper inheritance syntax with a valid parent type.

```markdown
### Test [Parent]  <!-- ✅ Valid: proper inheritance -->
- name: string
```

Or if no inheritance:

```markdown
### Test  <!-- ✅ Valid: no inheritance -->
- name: string
```

### Array Type Issues

#### ❌ Mixing Array Syntax Incorrectly

**Problem:** When using multiple types with arrays, the syntax must be consistent.

```markdown
### Test
- primitive: string[], integer, float, boolean  <!-- ❌ Invalid: mixing array and non-array types incorrectly -->
```

**Solution:** Use consistent array syntax or separate into different attributes.

```markdown
### Test
- strings: string[]      <!-- ✅ Valid: array of strings -->
- numbers: integer[]     <!-- ✅ Valid: array of integers -->
- single_value: string   <!-- ✅ Valid: single value -->
```

Or use union types properly:

```markdown
### Test
- values: string | integer | float  <!-- ✅ Valid: union type -->
```

## Quick Reference: Valid vs Invalid

| Issue                   | ❌ Invalid                                       | ✅ Valid                                           |
| ----------------------- | ----------------------------------------------- | ------------------------------------------------- |
| Name starts with number | `1User`, `123test`                              | `User1`, `test123`                                |
| Name with space         | `User Profile`, `user name`                     | `UserProfile`, `user_name`                        |
| Name with special char  | `User-Profile`, `user.name`                     | `UserProfile`, `user_name`                        |
| Missing type            | `- name`                                        | `- name: string`                                  |
| Undefined type          | `- profile: Profile` (if Profile doesn't exist) | `- profile: Profile` (if Profile exists)          |
| Duplicate object        | Two `### User` sections                         | Unique object names                               |
| Duplicate attribute     | Two `- name: string` in same object             | Unique attribute names                            |
| Empty model             | No objects defined                              | At least one object                               |
| Empty object            | `### User` with no attributes                   | `### User` with attributes or `allow_empty: true` |
| Empty XML option        | `- XML:` (empty)                                | `- XML: elementName`                              |
| XML wrapped depth       | `- XML: a/b/c/d` (3+ levels)                    | `- XML: a/b` (2 levels)                           |
| Reserved name           | `- __other__: string`                           | `- other: string`                                 |

## Tips for Avoiding Common Mistakes

1. **Always validate first**: Run `md-models validate -i model.md` before generating code
2. **Start simple**: Begin with basic types and simple structures, then add complexity
3. **Use descriptive names**: Follow naming conventions from the start to avoid refactoring later
4. **Check type references**: Ensure all referenced types exist before using them
5. **Test incrementally**: Add objects and attributes one at a time, validating as you go
6. **Read error messages carefully**: Validation errors include line numbers and specific solutions
7. **Use basic types when possible**: Prefer `string`, `number`, `integer`, `boolean` over custom types when appropriate
