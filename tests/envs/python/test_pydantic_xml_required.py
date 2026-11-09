"""Regression test for required attributes without a default (python-pydantic-xml).

The generator used to write `element(default, ...)` for such attributes, which
raised a NameError when the generated module was imported. Importing the module
is therefore the first check.
"""

from pydantic import ValidationError

from gen_pydantic_xml_required import Part, Sample  # type: ignore

REQUIRED = {"sample_id": "s1", "name": "x", "code": "C1"}

if __name__ == "__main__":
    sample = Sample(**REQUIRED, parts=[Part(label="a")])

    # Every required attribute without a default must be required.
    for missing in REQUIRED:
        kwargs = {k: v for k, v in REQUIRED.items() if k != missing}
        try:
            Sample(**kwargs)
        except ValidationError:
            continue
        raise AssertionError(f"'{missing}' should be required")

    try:
        Part()
    except ValidationError:
        pass
    else:
        raise AssertionError("'label' should be required")

    # Optional attributes still default to None.
    assert sample.note is None

    # XML round trip: `code` is an XML attribute, `name` an element.
    xml = sample.to_xml()
    assert b'code="C1"' in xml, xml
    back = Sample.from_xml(xml)
    assert (back.sample_id, back.name, back.code) == ("s1", "x", "C1")
    assert back.parts[0].label == "a"

    print("PyDantic XML required-attribute test successful")
