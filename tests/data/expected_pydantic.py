## This is a generated file. Do not modify it manually!

from __future__ import annotations
from pydantic import BaseModel, Field, ConfigDict
from typing import Optional
from enum import Enum
from uuid import uuid4
from datetime import date, datetime


class Test(BaseModel):
    name: str
    number: float = 1.0
    test2: list[Test2] = Field(default_factory=list)
    ontology: Optional[Ontology] = Field(default=None)

    # JSON-LD fields
    ld_id: str = Field(
        alias="@id",
        default_factory=lambda: "tst:Test/" + str(uuid4())
    )
    ld_type: list[str] = Field(
        alias="@type",
        default_factory = lambda: [
            "tst:Test",
        ],
    )
    ld_context: dict[str, str | dict] = Field(
        alias="@context",
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


class Test2(BaseModel):
    names: list[str] = Field(default_factory=list)
    number: Optional[float] = Field(default=None)

    # JSON-LD fields
    ld_id: str = Field(
        alias="@id",
        default_factory=lambda: "tst:Test2/" + str(uuid4())
    )
    ld_type: list[str] = Field(
        alias="@type",
        default_factory = lambda: [
            "tst:Test2",
        ],
    )
    ld_context: dict[str, str | dict] = Field(
        alias="@context",
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