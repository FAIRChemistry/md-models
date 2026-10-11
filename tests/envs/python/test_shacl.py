"""Validates JSON-LD from the generated pydantic models against the generated SHACL shapes."""

import json

import pyshacl
import rdflib

from gen_pydantic_jsonld import Molecule  # type: ignore
from test_pydantic_jsonld import BASE, build_document, to_graph

SHAPES = rdflib.Graph().parse("gen_shapes.ttl", format="turtle")


def validate(data_graph: rdflib.Graph) -> tuple[bool, str]:
    conforms, _, report = pyshacl.validate(data_graph, shacl_graph=SHAPES)
    return conforms, report


def assert_conforms(data_graph: rdflib.Graph):
    conforms, report = validate(data_graph)
    assert conforms, report


def test_valid_document_conforms():
    assert_conforms(to_graph(build_document()))


def test_empty_optional_lists_conform():
    document = build_document()
    document.molecules.append(Molecule(id="bare", name="Bare molecule"))
    assert_conforms(to_graph(document))


def test_whole_numbers_in_float_list_conform():
    # A JSON-LD processor types 1 and 2 as xsd:integer; they are still valid floats
    document = build_document()
    document.molecules[0].values = [1, 2]
    assert_conforms(to_graph(document))
    data = json.loads(document.model_dump_json(by_alias=True, exclude_none=True))
    data["molecules"][0]["values"] = [1, 2.5]
    assert_conforms(rdflib.Graph().parse(data=json.dumps(data), format="json-ld", base=BASE))


def test_missing_required_name_fails():
    data = json.loads(build_document().model_dump_json(by_alias=True, exclude_none=True))
    del data["molecules"][0]["name"]
    graph = rdflib.Graph().parse(data=json.dumps(data), format="json-ld", base=BASE)
    conforms, report = validate(graph)
    assert not conforms, "a molecule without name must not conform"
    assert "MinCountConstraintComponent" in report and "schema:name" in report, report


def test_wrong_list_item_datatype_fails():
    data = json.loads(build_document().model_dump_json(by_alias=True, exclude_none=True))
    data["molecules"][0]["values"].append("not a number")
    graph = rdflib.Graph().parse(data=json.dumps(data), format="json-ld", base=BASE)
    conforms, report = validate(graph)
    assert not conforms, "a string inside values must not conform"
    assert "OrConstraintComponent" in report and "values" in report, report


def test_unknown_enum_value_fails():
    data = json.loads(build_document().model_dump_json(by_alias=True, exclude_none=True))
    data["kind"] = "unknown"
    graph = rdflib.Graph().parse(data=json.dumps(data), format="json-ld", base=BASE)
    conforms, report = validate(graph)
    assert not conforms, "an unknown enum value must not conform"
    assert "InConstraintComponent" in report, report


if __name__ == "__main__":
    for name, test in list(globals().items()):
        if name.startswith("test_"):
            test()
    print("SHACL Test successful")
