"""End-to-end JSON-LD checks for pydantic models generated from tests/data/model_jsonld.md."""

import json

import rdflib
from pydantic import ValidationError
from rdflib.namespace import RDF, RDFS

from gen_pydantic_jsonld import Document, Molecule  # type: ignore

EX = rdflib.Namespace("https://example.org/jsonld/")
SCHEMA = rdflib.Namespace("https://schema.org/")
OBO = rdflib.Namespace("http://purl.obolibrary.org/obo/")
# Resolve relative IRIs against a file location, as a JSON-LD processor reading a file would
BASE = "file:///data/document.json"


def to_graph(model) -> rdflib.Graph:
    data = model.model_dump_json(by_alias=True, exclude_none=True)
    return rdflib.Graph().parse(data=data, format="json-ld", base=BASE)


def build_document() -> Document:
    molecule = Molecule(
        id="ManNAc",
        name="N-Acetylmannosamine",
        vessel_id="v0",
        references=["https://doi.org/10.1000/xyz"],
    )
    return Document(
        id="doc1",
        label="Test",
        homepage="https://example.org/home",
        molecules=[molecule],
    )


def test_version_default_and_pattern():
    assert Document(id="d").version == "2.0"
    try:
        Document(id="d", version="not-a-version")
    except ValidationError:
        return
    raise AssertionError("version pattern was not enforced")


def test_hash_prefix_expands():
    assert (None, RDFS.label, rdflib.Literal("Test")) in to_graph(build_document())


def test_frontmatter_underscore_prefix_expands():
    molecule = Molecule(id="w", name="water")
    molecule.add_type_term("CHEBI:15377")
    assert (None, RDF.type, OBO["CHEBI_15377"]) in to_graph(molecule)


def test_add_namespace_underscore_prefix_expands():
    molecule = Molecule(id="w", name="water")
    molecule.add_type_term("GO:0008150", "GO", "http://purl.obolibrary.org/obo/GO_")
    assert (None, RDF.type, OBO["GO_0008150"]) in to_graph(molecule)


def test_fields_without_term_are_in_graph():
    g = to_graph(build_document())
    assert (None, EX["Document/molecules"], None) in g
    assert (None, RDF.type, EX["Molecule"]) in g
    assert (None, EX["Document/version"], rdflib.Literal("2.0")) in g
    assert (None, EX["Molecule/id"], rdflib.Literal("ManNAc")) in g


def test_identifiers_are_literals():
    g = to_graph(build_document())
    assert (None, SCHEMA.identifier, rdflib.Literal("doc1")) in g
    assert (None, EX["Molecule/vessel_id"], rdflib.Literal("v0")) in g
    assert not [t for t in g if any(str(term).startswith("file:") for term in t)]


def test_iri_fields_are_iris():
    g = to_graph(build_document())
    assert (None, SCHEMA.url, rdflib.URIRef("https://example.org/home")) in g
    assert (None, SCHEMA.citation, rdflib.URIRef("https://doi.org/10.1000/xyz")) in g


def roundtrip(document: Document) -> tuple[dict, dict]:
    written = json.loads(document.model_dump_json(by_alias=True, exclude_none=True))
    reread = Document.model_validate(written)
    return written, json.loads(reread.model_dump_json(by_alias=True, exclude_none=True))


def test_roundtrip_keeps_ids():
    written, rewritten = roundtrip(build_document())
    assert rewritten["@id"] == written["@id"]
    assert rewritten["molecules"][0]["@id"] == written["molecules"][0]["@id"]


def test_roundtrip_keeps_extra_types():
    document = build_document()
    document.molecules[0].add_type_term("CHEBI:15377")
    _, rewritten = roundtrip(document)
    assert rewritten["molecules"][0]["@type"] == ["ex:Molecule", "CHEBI:15377"]
    _, rewritten_twice = roundtrip(Document.model_validate(rewritten))
    assert rewritten_twice == rewritten


def test_read_repairs_old_context_and_keeps_extra_prefixes():
    written = json.loads(build_document().model_dump_json(by_alias=True, exclude_none=True))
    written["@context"]["id"] = {"@id": "schema:identifier", "@type": "@id"}  # pre-fix context
    written["@context"]["myprefix"] = "https://my.example/"
    reread = Document.model_validate(written)
    assert reread.ld_context["id"] == "schema:identifier"
    assert reread.ld_context["myprefix"] == "https://my.example/"


if __name__ == "__main__":
    for name, test in list(globals().items()):
        if name.startswith("test_"):
            test()
    print("PyDantic JSON-LD Test successful")
