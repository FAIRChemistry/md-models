from gen_pydantic import Test, Test2, Ontology  # type: ignore
from uuid import UUID


def is_valid_uuid(uuid_str: str) -> bool:
    try:
        # Extract UUID from the full string (tst:Test/UUID or tst:Test2/UUID)
        uuid_part = uuid_str.split("/")[-1]
        UUID(uuid_part)
        return True
    except ValueError:
        return False


if __name__ == "__main__":
    test = Test(name="test", number=1, ontology=Ontology.ECO)
    test.add_to_test2(names=["test2"], number=2)

    test2 = Test2(names=["test2"], number=2)
    test.test2.append(test2)

    result = test.model_dump(by_alias=True)

    # Verify structure while allowing for dynamic UUIDs
    assert is_valid_uuid(result["@id"])
    assert result["@type"] == ["tst:Test"]
    assert result["@context"] == {
        "tst": "https://www.github.com/my/repo/",
        "schema": "http://schema.org/",
        "name": {
            "@id": "schema:hello",
            "@type": "@id",
        },
        "number": "schema:one",
        "test2": "schema:something",
    }
    assert result["name"] == "test", "expected name to be test"
    assert result["number"] == 1, "expected number to be 1"
    assert len(result["test2"]) == 2, "expected test2 to be a list with one object"

    test2_obj = result["test2"][0]
    assert is_valid_uuid(test2_obj["@id"]), "expected test2 id to be a valid UUID"
    assert test2_obj["@type"] == ["tst:Test2"], "expected test2 type to be tst:Test2"
    assert test2_obj["names"] == ["test2"], "expected names to be test2"
    assert test2_obj["number"] == 2, "expected number to be 2"
    assert result["ontology"].value == "https://www.evidenceontology.org/term/", (
        "expected ontology to be ECO, but got " + result["ontology"]
    )

    test2_obj = result["test2"][1]
    assert is_valid_uuid(test2_obj["@id"]), "expected test2 id to be a valid UUID"
    assert test2_obj["@type"] == ["tst:Test2"], "expected test2 type to be tst:Test2"
    assert test2_obj["names"] == ["test2"], "expected names to be test2"
    assert test2_obj["number"] == 2, "expected number to be 2"

    print("PyDantic Test successful")
