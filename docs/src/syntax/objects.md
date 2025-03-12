# Objects

Objects are the building blocks of your data structure. Think of them as containers for related information, similar to how a form organizes different fields of information about a single topic.

## What is an Object?

An object is simply a named collection of properties. For example, a `Person` object might have properties like `name`, `age`, and `address`. In our system, objects are defined using a straightforward format that's easy to read and write, even if you're not a programmer.

## How to Define an Object

You start objects by declaring its name using a level 3 heading (`###`) followed by the name of the object. In the example below, we define an object called `Person`.

```markdown
### Person

This is an object definition.
```

Great! Now we have a named object. But what's next?

## Object Properties

Objects can have properties, which define the specific data fields that belong to the object. Properties are defined using a structured list format with the following components:

1. The property name - starts with a dash (`-`) followed by the name
2. The property type - indicates what kind of data the property holds
3. Optional metadata - additional specifications like descriptions, constraints, or validation rules

Here's the basic structure:

```markdown
### Person (schema:object)

- name
  - type: string
  - description: The name of the person
```

Lets break this down:

- `- name` - The name of the property
- `- type: string` - The type of the property, because we expect a name to be a string (e.g. "John Doe")
- `- description: The name of the person` - A description of the property

The name of the property and its type are required. The description is optional, but it is a good practice to add it. Later on we will see that a thourough description can be used to guide a large language model to extract the information from a text.

> By default, properties are optional. If you want to make a property required, you need to bold the property name using either `__name__` or `**name**`. Replace `name` with the name of the property.

### Property Types

The data type of a property is very important and generally communicates what kind of data the property holds. Here is a list of the supported base types:

- `string` - A string of characters
- `integer` - A whole number
- `float` - A floating point number
- `number` - A numeric value (integer or float)
- `boolean` - A true or false value

### Arrays

While these types are the building blocks, they fail to capture the full range of data types that can be used in a data model. For example, we need to be able to express that a property is an array/list of strings, or an array/list of numbers. This is where the `array` notation comes in.

We define an array of a given type by placing empty square brackets after the type. For example, an array of strings would be written as `string[]`[^inspired by TypeScript].

```markdown
### Person (schema:object)

- an_array_of_strings
  - type: string[]
  - description: An array of strings
- an_array_of_numbers
  - type: number[]
  - description: An array of numbers
```

### Connecting Objects

Now we know how to define singular and array properties, but we often need to create relationships between objects in our data models. For example, a `Person` object might have an `address` property that references an `Address` object. This relationship is easily established by using another object's name as a property's type.

```markdown
### Person

- name
  - type: string
- address
  - type: Address

### Address

- street
  - type: string
- city
  - type: string
- zip
  - type: string
```

This approach allows you to build complex, interconnected data models that accurately represent real-world relationships between entities. You can create both one-to-one relationships (like a person having one address) and one-to-many relationships (by using array notation).
