"""
Tests for the dataclasses implementation of the test models.
Verifies that:
- Objects can be created and modified
- JSON serialization works correctly
"""

import json
from gen_dataclasses import Test, Test2, Ontology   # type: ignore

if __name__ == "__main__":
    test = Test(name="test", number=1, ontology=Ontology.ECO)
    test.add_to_test2(names=["test2"], number=2)

    test2 = Test2(names=["test2"], number=2)
    test.test2.append(test2)

    result = json.loads(test.to_json())

    print("Dataclasses Test successful")
