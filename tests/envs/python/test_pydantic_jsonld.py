"""End-to-end JSON-LD checks for pydantic models generated from tests/data/model_jsonld.md."""

import datetime
import json

import rdflib
from pydantic import ValidationError
from rdflib.collection import Collection
from rdflib.namespace import RDF, RDFS, XSD

from gen_pydantic_jsonld import Document, Kind, Item  # type: ignore

EX = rdflib.Namespace("https://example.org/jsonld/")
SCHEMA = rdflib.Namespace("https://schema.org/")
EXT = rdflib.Namespace("https://example.org/")
# Resolve relative IRIs against a file location, as a JSON-LD processor reading a file would
BASE = "file:///data/document.json"


def to_graph(model) -> rdflib.Graph:
    data = model.model_dump_json(by_alias=True, exclude_none=True)
    return rdflib.Graph().parse(data=data, format="json-ld", base=BASE)


def build_document() -> Document:
    item = Item(
        id="i1",
        name="First item",
        owner_id="v0",
        references=["https://doi.org/10.1000/xyz"],
        values=[1.0, 0.5, 0.5, 0.25],
    )
    return Document(
        id="doc1",
        label="Test",
        homepage="https://example.org/home",
        items=[item],
        created=datetime.date(2024, 5, 17),
        kind=Kind.DRAFT,
    )


def list_items(g: rdflib.Graph, predicate: rdflib.URIRef) -> list:
    """Items of the single rdf:List that is the object of `predicate`."""
    (head,) = list(g.objects(None, predicate))
    return list(Collection(g, head))


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
    item = Item(id="w", name="sample")
    item.add_type_term("cat:42")
    assert (None, RDF.type, EXT["category_42"]) in to_graph(item)


def test_add_namespace_underscore_prefix_expands():
    item = Item(id="w", name="sample")
    item.add_type_term("tag:7", "tag", "https://example.org/tag_")
    assert (None, RDF.type, EXT["tag_7"]) in to_graph(item)


def test_fields_without_term_are_in_graph():
    g = to_graph(build_document())
    assert (None, EX["Document/items"], None) in g
    assert (None, RDF.type, EX["Item"]) in g
    assert (None, EX["Document/version"], rdflib.Literal("2.0")) in g
    assert (None, EX["Item/id"], rdflib.Literal("i1")) in g


def test_identifiers_are_literals():
    g = to_graph(build_document())
    assert (None, SCHEMA.identifier, rdflib.Literal("doc1")) in g
    assert (None, EX["Item/owner_id"], rdflib.Literal("v0")) in g
    assert not [t for t in g if any(str(term).startswith("file:") for term in t)]


def test_iri_fields_are_iris():
    g = to_graph(build_document())
    assert (None, SCHEMA.url, rdflib.URIRef("https://example.org/home")) in g
    assert list_items(g, SCHEMA.citation) == [rdflib.URIRef("https://doi.org/10.1000/xyz")]


def test_arrays_are_ordered_lists_keeping_duplicates():
    items = list_items(to_graph(build_document()), EX["Item/values"])
    assert [float(x) for x in items] == [1.0, 0.5, 0.5, 0.25]


def test_date_is_typed_literal():
    g = to_graph(build_document())
    assert (None, EX["Document/created"], rdflib.Literal("2024-05-17", datatype=XSD.date)) in g


def roundtrip(document: Document) -> tuple[dict, dict]:
    written = json.loads(document.model_dump_json(by_alias=True, exclude_none=True))
    reread = Document.model_validate(written)
    return written, json.loads(reread.model_dump_json(by_alias=True, exclude_none=True))


def test_roundtrip_keeps_ids():
    written, rewritten = roundtrip(build_document())
    assert rewritten["@id"] == written["@id"]
    assert rewritten["items"][0]["@id"] == written["items"][0]["@id"]


def test_roundtrip_keeps_extra_types():
    document = build_document()
    document.items[0].add_type_term("cat:42")
    _, rewritten = roundtrip(document)
    assert rewritten["items"][0]["@type"] == ["ex:Item", "cat:42"]
    _, rewritten_twice = roundtrip(Document.model_validate(rewritten))
    assert rewritten_twice == rewritten


def test_read_repairs_old_context_and_keeps_extra_prefixes():
    written = json.loads(build_document().model_dump_json(by_alias=True, exclude_none=True))
    written["@context"]["id"] = {"@id": "schema:identifier", "@type": "@id"}  # pre-fix context
    written["@context"]["myprefix"] = "https://my.example/"
    reread = Document.model_validate(written)
    assert reread.ld_context["id"] == "schema:identifier"
    assert reread.ld_context["myprefix"] == "https://my.example/"


def written_document() -> dict:
    return json.loads(build_document().model_dump_json(by_alias=True, exclude_none=True))


def test_read_bare_string_type():
    written = written_document()
    written["@type"] = "schema:Thing"
    assert Document.model_validate(written).ld_type == ["ex:Document", "schema:Thing"]


def test_read_remote_string_context():
    written = written_document()
    written["@context"] = "https://schema.org"
    assert Document.model_validate(written).ld_context == Document(id="d").ld_context


def test_read_list_context():
    written = written_document()
    written["@context"] = ["https://schema.org", {"myprefix": "https://my.example/", "id": "ex:wrong"}]
    context = Document.model_validate(written).ld_context
    assert context["myprefix"] == "https://my.example/"
    assert context["id"] == "schema:identifier"


def test_read_null_id_gets_fresh_id():
    written = written_document()
    written["@id"] = None
    assert Document.model_validate(written).ld_id.startswith("ex:Document/")


if __name__ == "__main__":
    for name, test in list(globals().items()):
        if name.startswith("test_"):
            test()
    print("PyDantic JSON-LD Test successful")
