## This is a generated file. Do not modify it manually!

from __future__ import annotations
from typing import Dict, List, Optional
from uuid import uuid4
from datetime import date, datetime

from lxml.etree import _Element
from pydantic import PrivateAttr, model_validator
from pydantic_xml import attr, element

import sdRDM
from sdRDM.base.listplus import ListPlus
from sdRDM.tools.utils import elem2dict


class Test(
    sdRDM.DataModel,
    search_mode="unordered",
):
    name: str = attr(
            tag="name",
            json_schema_extra=dict(term = "schema:hello",)
        )

    number: Optional[float] = attr(
            default=None,
            tag="number",
            json_schema_extra=dict(term = "schema:one",)
        )

    test2: List[Test2] = element(
            default_factory=ListPlus,
            tag="SomeTest2",
            json_schema_extra=dict(term = "schema:something",)
        )

    ontology: Optional[Ontology] = element(
            default=None,
            tag="ontology",
            json_schema_extra=dict()
        )

    _repo: str = PrivateAttr(default="https://www.github.com/my/repo/")

    
    def add_to_test2(
        self,
        names: List[str],
        number: Optional[float],
        **kwargs,
    ):
        params = {
            
            "names": names, 
            "number": number
        }

        self.test2.append(
            Test2(**params)
        )

        return self.test2[-1]

class Test2(
    sdRDM.DataModel,
    search_mode="unordered",
):
    names: List[str] = element(
            default_factory=ListPlus,
            tag="name",
            json_schema_extra=dict(term = "schema:hello",)
        )

    number: Optional[float] = attr(
            default=None,
            tag="number",
            json_schema_extra=dict(term = "schema:one",minimum = "0",)
        )

    _repo: str = PrivateAttr(default="https://www.github.com/my/repo/")

    

class Ontology(Enum):
    ECO = "https://www.evidenceontology.org/term/"
    GO = "https://amigo.geneontology.org/amigo/term/"
    SIO = "http://semanticscience.org/resource/"