# Introduction

MD-Models is a markdown-based specification language for research data management.

It is designed to be easy to read and write, and to be converted to various programming languages and schema languages.

```markdown
# Hello MD-Models

This is a simple markdown file that defines a model.

### Object

Enrich your objects with documentation and communicate intent to domain experts.

This is a simple object definition:

- string_attribute
    - type: string
    - description: A string attribute
- integer_attribute
    - type: integer
    - description: An integer attribute
```

## Core Philosophy

The primary motivation behind MD-Models is to reduce cognitive overhead and maintenance burden by unifying documentation and structural definition into a single source of truth. Traditional approaches often require maintaining separate artifacts:

1. Technical schemas (JSON Schema, XSD, ShEx, SHACL)
2. Programming language implementations
3. Documentation for domain experts
4. API documentation

This separation frequently leads to documentation drift and increases the cognitive load on both developers and domain experts.

### A Little Anecdote

When I began my journey in research data management, I was frequently overwhelmed by the intricate tools and standards in use. As a researcher suddenly thrown into a blend of software engineering, format creation, and data management, it felt like I was plunged into deep water without a safety net.

Data management, by its very nature, spans multiple disciplines and demands a thorough understanding of the domain, the data itself, and the available tools. Yet, even the most impressive tools lose their value if they don’t cater to the needs of domain experts. I came to realize that those experts are best positioned to define the structure and purpose of the data, but the overwhelming complexity of existing tools and standards often prevents their active participation.

MD-Models is my response to this challenge. It makes building structured data models easier by enabling domain experts to document the data’s intent and structure in a clear and manageable way. Markdown is an ideal choice for this task. It is simple to read and write, and it effectively communicates the necessary intent. Moreover, its semi-structured format allows for effortless conversion into various schema languages and programming languages, eliminating the need for excessive boilerplate code.
