---
hide:
  - navigation
---


# Syntax

This guide outlines how to document a data model schema using MD-Models. The goal is to explain the structure and key components of the data model in clear, non-technical language to make it accessible to users who may not have a background in programming. This documentation will also ensure that the data model is organized, easy to maintain, and compatible with various external systems.

<details><summary>Full example</summary>

```markdown
---
repo: https://myrepo.com/datamodel
prefix: myapp
prefixes:
  schema: schema.org
---

### User (schema:Person)

This type defines a user profile.

- name
  - Type: string
  - Description: The full name of the user.
  - Term: schema:name
- address
  - Type: Address
  - Description: The user's address.
- orders
  - Type: Order[]
  - Description: Complete list of the user's orders.

### Address

This object represents a user's address.

- street
    - Type: string
    - Description: The street of a user
    - Term: schema:streetAddress
- postal_code
    - Type: string
    - Description: The postal code of a user
    - Term: schema:postalCode
  
### Order

This object defines an order.

- product_id
  - Type: string
  - Description: The ID of the product that has been ordered
- status
  - Type: OrderStatus
  - Description: The current state of the user's order.

### OrderStatus

This enumeration encodes the possible states an order can be in.

DELIVERED = "delivered"
IN_TRANSIT = "in_transit"
OPEN = "open"
```

</details>

## YAML Frontmatter: Metadata Definition

At the very beginning of the document, we define what’s called the YAML frontmatter. This section provides important metadata (background information) that gives context to the entire data model. Think of it as the “settings” for your documentation. This metadata helps systems interpret the model correctly, especially if the model needs to work with other systems or tools.

**Why is the Frontmatter Important?**

The frontmatter provides key information that serves multiple purposes:

- Repository Information: Where is the data model stored? This could be a web address (URL) to a source repository where the model is maintained.
- Prefix for XML Serialization: If your data model is going to be converted into XML format, it’s essential to define a prefix that ensures each part of the model is properly named and understood by machines.
- Other Prefixes: You might need to reference common external vocabularies (like schema.org) to make your model interoperable with other data formats.

Example of Frontmatter:

```yaml
---
repo: https://myrepo.com/datamodel
prefix: myapp
prefixes:
  schema: schema.org
---
```

Explanation:

- `repo`: This is a web address (URL) that points to the repository where the data model is stored. In this example, it’s https://myrepo.com/datamodel.
- `prefix`: This is the default prefix (myapp) used when the model is serialized into XML. This prefix ensures the data elements are uniquely named.
- `prefixes`: This section maps additional prefixes to well-known external namespaces, such as schema mapped to schema.org. This helps define standardized data fields like schema:name for names, or schema:address for addresses.

## General Description of the Data Model

This section is defined by a level 1 heading (#) and gives an overall summary of the data model. It explains what the data model is about and why it exists, providing a general overview that someone unfamiliar with the model can understand.

**Why is the General Description Important?**

The general description serves as an introduction to the data model. It should provide a high-level explanation that answers:

- What is the purpose of the data model?
- What kind of data or entities does it represent?
- Who would use this model and for what purpose?

This section does not need to be too technical. Its 
purpose is to help everyone, regardless of technical background, understand what the model is designed to do.

Example of a General Description:

```markdown
# My Application Data Model

This data model defines and describes the key entities and relationships within My Application. The model focuses on users, their addresses, and the various statuses that a user account can have. It provides a structured way to organize and manage data, ensuring consistency across the system and compatibility with external systems like XML and schema.org.

Explanation:

- Entities: In this case, the entities are users, addresses, and account statuses.
- Purpose: The model provides a way to organize and manage data, ensuring that the system operates consistently and integrates easily with other systems.
```

This general description helps someone unfamiliar with the model quickly understand what kind of data is being managed and why.

## Object Definitions

Objects are the building blocks of the data model. They represent the things or entities within the system that you are tracking or managing. For example, objects can represent people, products, locations, or any other type of entity relevant to your application.

**What is an Object?**

An object in the context of a data model is something that you want to store information about. It could be a user, a product, a service, or even something abstract like an event or a status. Objects are usually defined by a name and a set of attributes (properties). Within a markdown data model, an object is defined by a level 3 heading (###).

Example of an Object:

```markdown
### User (schema:Person)

- name
  - Type: string
```

- Object name: `User` – This represents an individual who uses the application.
- Namespace reference: `schema:Person` This maps the object to the standardized `Person` term from schema.org. By doing this, you ensure that external systems will recognize and understand the object when interacting with your model.

**Best Practices for Object Names**

- Object names should be capitalized and written in PascalCase: This means that each word in the object name should start with a capital letter, and there should be no spaces or underscores between words. This makes object names clear and easy to read.

### Attributes

Attributes define the characteristics or properties of an object. For example, a `User` object might have attributes like `name`, `email_address`, and `age` These attributes describe specific information about the user. Within markdown data models, an attribute is defined as a list element (-) and has to appear under the object it belongs to.

Example of attributes for the `User` Object:

```markdown
- **name**
  - Type: string
  - Description: The full name of the user.
  - Term: schema:name
- age
  - Type: number
  - Description: The age of the user in years.
  - Term: schema:age
```

Explanation:

- `name`: This attribute stores the user’s name. It is a string (a sequence of characters), and it is mapped to the schema:name term from schema.org. By enclosing the attribute name in double asterisks (\*\*), we make it bold and thus required.
- `age`: This attribute stores the user’s age as a number. It uses the schema:age term, ensuring consistency with other systems that also use this standard.

**Best Practices for attribute names**

- Do not use special characters or numbers at the start of attribute names: This ensures that the names are valid and won’t cause errors when interacting with different systems.
- Avoid spaces in attribute names: Instead, use underscores (`_`) to separate words. This ensures that the names are easy to read and consistent.

### Configuring attributes

You may have noticed that attributes contain a sub-list of key-value pairs. These are used to define specific details about each attribute. You can insert whatever metadata you'd like to add, but there are two mandatory ones:

- `Type`: The data-type associated with the attribute. This is vital to check if the values given to an attribute are as expected. For example, a user’s age should be a number, not a string.
- `Description`: A short description of what the attribute represents. This helps others understand the purpose of the attribute and how it should be used. In addition, these descriptions are helpful to assist OpenAI-based structured respones to extract metadata from text data.

Aside of these mandatory optionals, here are some optional ones:

- `XML`: This alias will be used upon XML handling. It can be particularly useful in cases where the JSON snake case notation violates the XML naming conventions.
- `Term`: This is where you can add ontology terms to identify the semantic meaning of an attribute. You can enter a valid URI or make use of prefixed shortforms such as `schema:person`. When using the latter, make sure that you have defined the prefix within the YAML frontmatter.

Since markdown data models are a superset of JSON-Schema, you can use any of the JSON-Schema attributes to further define the behavior of your attributes. This includes options like `default`, `minimum`, `example` and more.

### Using Types

The `Type` option of an attribute is a powerful tool to ensure data consistency and accuracy. By specifying the data type of an attribute, you can prevent incorrect values from being entered and ensure that the data is always in the expected format. These are the currently supported base types:

- `string`- Text data, such as names, addresses, and descriptions.
- `number`- Numerical data, such as a length, prices, and quantities. Resolves to `float`
- `float` - Explicit floating point number type.
- `integer` - Whole number type, such as ages, counts or IDs.
- `boolean` - Truth values that can be either `true`or `false`
- `bytes` - Raw byte data such as files or images.

In addition to basic types, you can also specify the usage of another object that you have defined within your data model. For instance, if there is an `Address` object present, you can specify that the `address` attribute of a `User` object should be of type `Address`.

When dealing with array/list data, you can tranform any type into an array by adding `[]` to the end of the type. For example, `string[]` would represent an array of strings. Same applies to complex types (other objects in your model) as well.

## Enumerations

Enumerations are used when an attribute can only have a specific set of predefined values. These values are usually constant and represent different states or categories. For example, a Status enumeration could represent whether a user’s account is active, inactive, or suspended.

**What are Enumerations?**

An enumeration is a list of possible values that an attribute can take. This is especially useful for fields like account status, where only a limited number of values are valid. By using enumerations, you ensure that the data entered is always within the allowed range, reducing errors and improving consistency.

Example of an Enumeration:

```markdown
### Status

Represents the possible states of a user account.

ACTIVE = "active"
INACTIVE = "inactive"
SUSPENDED = "suspended"
```

Explanation:

- The Status enumeration defines three possible values that a status attribute can have:
  - `ACTIVE`: The account is active and operational.
  - `INACTIVE`: The account is inactive, possibly due to inactivity or manual suspension.
  - `SUSPENDED`: The account is temporarily or permanently suspended.

By defining enumerations, you restrict the attribute values to valid options, ensuring that no invalid data is entered.

## Namespaces and Prefixes

Namespaces and prefixes provide a way to align your data model with external standards and ensure that your data is compatible with other systems. A namespace acts as a reference to a specific domain of terms, helping avoid conflicts or confusion when similar terms exist in different systems.

**Why are Namespaces and Prefixes Important?**

When you’re building a data model that may need to interact with other systems, it’s important to reference standardized terms. Namespaces allow you to link the terms in your model to well-known vocabularies (like `schema.org`) so that external systems can recognize and understand your data. By doing this, you make your data model interoperable.

Example of prefixes:

```yaml
prefixes:
  schema: schema.org
```

Explanation:

- The schema prefix is mapped to schema.org. This means that any term in your model prefixed with schema (like `schema:name` or `schema:address`) is directly linked to a globally recognized vocabulary. This helps standardize your data and makes it easier to integrate with other platforms or services.

## Full example

In this section you can find a full example that describes a simple data model for a user profile. This model includes the user’s name, email address, age, and status. It also defines an enumeration for the user status attribute.

<details><summary>Click to show</summary>

```markdown
---
repo: https://myrepo.com/datamodel
prefix: myapp
prefixes:
  schema: schema.org
---

### User (schema:Person)

This type defines a user profile.

- name
  - Type: string
  - Description: The full name of the user.
  - Term: schema:name
- address
  - Type: Address
  - Description: The user's address.
- orders
  - Type: Order[]
  - Description: Complete list of the user's orders.

### Address

This object represents a user's address.

- street
    - Type: string
    - Description: The street of a user
    - Term: schema:streetAddress
- postal_code
    - Type: string
    - Description: The postal code of a user
    - Term: schema:postalCode
  
### Order

This object defines an order.

- product_id
  - Type: string
  - Description: The ID of the product that has been ordered
- status
  - Type: OrderStatus
  - Description: The current state of the user's order.

### OrderStatus

This enumeration encodes the possible states an order can be in.

DELIVERED = "delivered"
IN_TRANSIT = "in_transit"
OPEN = "open"
```

</details>

## Best Practices

To ensure your data model is clear, well-organized, and compatible with different systems, follow these best practices.

### Object Names

Object names should be capitalized and written in PascalCase. This means the first letter of each word is capitalized, and there are no spaces between words. This naming convention improves readability and keeps the documentation consistent. Names should also not start with numbers or special characters.

✅ Valid Example:

```markdown
### UserOrder
```

⛔️ Invalid Examples:

```markdown
### User Order

### 0UserOrder
```

### Attribute Names

- Attribute names must not start with numbers or special characters. This ensures compatibility with programming languages and data serialization formats.
- Avoid spaces in attribute names. Instead, use underscores (`_`) to separate words.

✅ Valid Example:

```markdown
- total_price
  - Type: number
  - Description: The total price of the order.
```

⛔️ Invalid Examples:

```markdown
- total price
  - Type: number
  - Description: The total price of the order.
- 3total price
  - Type: number
  - Description: The total price of the order.
```

### Descriptions for Objects and Attributes

Always include descriptions for both objects and attributes. These descriptions provide valuable context, helping both human readers and systems (like AI tools) understand the meaning and purpose of each object or attribute.

✅ Valid Example:

```markdown
### Product (schema:Product)

Represents an item that is available for purchase in the system.

- name
  - Type: string
  - Description: The name of the product.
```

⚠️ Bad Practice Example:

```markdown
### Product

- name
  - Type: string
```
