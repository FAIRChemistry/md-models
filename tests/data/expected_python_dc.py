"""
This file contains dataclass definitions for data validation.

Dataclasses are a built-in Python library that provides a way to define data models
with type hints and automatic serialization to JSON.

Usage example:
```python
from my_model import MyModel

# Validates data at runtime
my_model = MyModel(name="John", age=30)

# Type-safe - my_model has correct type hints
print(my_model.name)
```

For more information see:
https://docs.python.org/3/library/dataclasses.html

WARNING: This is an auto-generated file.
Do not edit directly - any changes will be overwritten.
"""


## This is a generated file. Do not modify it manually!

from __future__ import annotations
from dataclasses import dataclass, field
from dataclasses_json import config, dataclass_json
from typing import List, Optional, Union
from enum import Enum
from uuid import uuid4
from datetime import date, datetime


@dataclass_json
@dataclass
class Test:
    name: str = "2"
    number: Union[None,float,str] = 1
    test2: list[Test2] = field(default_factory=list)
    ontology: Optional[Ontology] = field(default=None, metadata=config(exclude=lambda x: x is None))

    # JSON-LD fields
    id: str = field(
        metadata=config(field_name="@id"),
        default_factory=lambda: "tst:Test/" + str(uuid4())
    )
    __type__: list[str] = field(
        metadata=config(field_name="@type"),
        default_factory = lambda: [
            "tst:Test",
        ],
    )
    __context__: dict[str, str | dict] = field(
        metadata=config(field_name="@context"),
        default_factory = lambda: {
            "tst": "https://www.github.com/my/repo/",
            "schema": "http://schema.org/",
            "name": {
                "@id": "schema:hello",
                "@type": "@id",
            },
            "number": "schema:one",
            "test2": "schema:something",
        }
    )


    def add_to_test2(
        self,
        names: list[str]= [],
        number: Optional[float]= None,
        **kwargs,
    ):
        params = {
            "names": names,
            "number": number
        }

        if "id" in kwargs:
            params["id"] = kwargs["id"]

        self.test2.append(
            Test2(**params)
        )

        return self.test2[-1]


@dataclass_json
@dataclass
class Test2:
    names: list[str] = field(default_factory=list)
    number: Optional[float] = field(default=None, metadata=config(exclude=lambda x: x is None))

    # JSON-LD fields
    id: str = field(
        metadata=config(field_name="@id"),
        default_factory=lambda: "tst:Test2/" + str(uuid4())
    )
    __type__: list[str] = field(
        metadata=config(field_name="@type"),
        default_factory = lambda: [
            "tst:Test2",
        ],
    )
    __context__: dict[str, str | dict] = field(
        metadata=config(field_name="@context"),
        default_factory = lambda: {
            "tst": "https://www.github.com/my/repo/",
            "schema": "http://schema.org/",
            "names": "schema:hello",
            "number": "schema:one",
        }
    )


class Ontology(Enum):
    ECO = "https://www.evidenceontology.org/term/"
    GO = "https://amigo.geneontology.org/amigo/term/"
    SIO = "http://semanticscience.org/resource/"