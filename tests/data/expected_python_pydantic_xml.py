## This is a generated file. Do not modify it manually!

from __future__ import annotations
from typing import Dict, List, Optional
from uuid import uuid4
from datetime import date, datetime
from xml.dom import minidom

from lxml.etree import _Element
from pydantic import PrivateAttr, model_validator
from pydantic_xml import attr, element, BaseXmlModel


class Test(
    BaseXmlModel,
    search_mode="unordered",
):
    name: str = attr(
            tag="name",
            json_schema_extra=dict(term = "schema:hello",)
        )

    number: Optional[float] = attr(
            default=1.0,
            tag="number",
            json_schema_extra=dict(term = "schema:one",)
        )

    test2: list[Test2] = element(
            default_factory=list,
            tag="SomeTest2",
            json_schema_extra=dict(term = "schema:something",)
        )

    ontology: Optional[Ontology] = element(
            default=None,
            tag="ontology",
            json_schema_extra=dict()
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

        self.test2.append(
            Test2(**params)
        )

        return self.test2[-1]

    def xml(self, encoding: str = "unicode") -> str | bytes:
        """Converts the object to an XML string

        Args:
            encoding (str, optional): The encoding to use. If set to "bytes", will return a bytes string.
                                      Defaults to "unicode".
        """

        if encoding == "bytes":
            return self.to_xml()

        raw_xml = self.to_xml(encoding=None)
        parsed_xml = minidom.parseString(raw_xml)
        return parsed_xml.toprettyxml(indent="  ")


class Test2(
    BaseXmlModel,
    search_mode="unordered",
):
    names: list[str] = element(
            default_factory=list,
            tag="name",
            json_schema_extra=dict(term = "schema:hello",)
        )

    number: Optional[float] = attr(
            default=None,
            tag="number",
            json_schema_extra=dict(term = "schema:one",minimum = "0",)
        )


    def xml(self, encoding: str = "unicode") -> str | bytes:
        """Converts the object to an XML string

        Args:
            encoding (str, optional): The encoding to use. If set to "bytes", will return a bytes string.
                                      Defaults to "unicode".
        """

        if encoding == "bytes":
            return self.to_xml()

        raw_xml = self.to_xml(encoding=None)
        parsed_xml = minidom.parseString(raw_xml)
        return parsed_xml.toprettyxml(indent="  ")


class Ontology(Enum):
    ECO = "https://www.evidenceontology.org/term/"
    GO = "https://amigo.geneontology.org/amigo/term/"
    SIO = "http://semanticscience.org/resource/"