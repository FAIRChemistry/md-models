# Descriptions

This section further highlights the usage of descriptions in MD-Models. Since we are using markdown, we can enrich our data model with any additional information that we want to add. This not only includes text, but also links and images.

## Text

To add a text description to an object, we can use the following syntax:

```markdown
### Product

A product is a physical or digital item that can be bought or sold.

- name
  - type: string
  - description: The name of the product
```

## Links

To add a link to an object, we can use the following syntax:

```markdown
### Product

[Additional information](https://www.google.com)

- name
  - type: string
  - description: The name of the product
```

## Images

To add an image to an object, we can use the following syntax:

```markdown
### Product

![Product image](https://www.google.com/images/branding/googlelogo/1x/googlelogo_color_272x92dp.png)

- name
  - type: string
  - description: The name of the product
```

> Please note that tables can be used within object definitions, but can under circumstances lead to parsing errors. It is therefore recommended to only use tables in sections.

## Sections

Since objects and enumerations can get quite complex, we can use sections to group related information together. The level 2 heading (`##`) can be used to create a new section:

```markdown
## Store-related information

This is section contains information about the store.

### Product

[...]

### Customer

[...]

## Sales-related information

This section contains information about the sales.

### Order

[...]

### Invoice

[...]
```

Within these sections, you can add any of the previously mentioned elements, including tables. This is very useful to breathe life into your data model and communicate intent and additional information. Treat this as the non-technical part you would usually add in an additional document. It should be noted, that the parsers will ignore these sections, so they will not be included in the generated code.

## Best Practices

- Use sections to group related information together.
- Use links to reference external sources.
- Use images to visually represent complex concepts.
- Use tables to represent concepts that are better understood in a table format.
