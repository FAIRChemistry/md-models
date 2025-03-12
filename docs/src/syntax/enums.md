# Enumerations

Sometimes you want to restrict the values that can be assigned to a property. For example, you might want to restrict the categories of a product to a set of predefined values. A product might be of category `book`, `movie`, `music`, or `other`. This is where enumerations come in.

## Defining an enumeration

To define an enumeration, we start the same as we do for any other type, by using a level 3 heading (###) and then the name of the type.

```markdown
### ProductCategory

BOOK = "book"
MOVIE = "movie"
MUSIC = "music"
OTHER = "other"
```

We are defining a key and value here, where the value is the actual value of the enumeration and the key is an identifier. This is required, because when we want to re-use the enumeration in a programming language, we need to be able to refer to it by a key. For instance, in python we can pass an enumeration via the following code:

```python
from model import ProductCategory, Product

product = Product(
    name="Inception",
    category=ProductCategory.MOVIE
)

print(product)
```

```json
{
    "name": "Inception",
    "category": "movie"
}
```

Similar to how we can use an object as a type for a property, we can also use an enumeration as a type for a property:

```markdown
### Product

- name
  - type: string
- category
  - type: ProductCategory
```
