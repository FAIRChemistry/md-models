from gen_pydantic_xml import Test, Test2, Ontology  # type: ignore

if __name__ == "__main__":
    test = Test(name="test", number=1, ontology=Ontology.ECO)
    test.add_to_test2(names=["test2"], number=2)

    test2 = Test2(names=["test2"], number=2)
    test.test2.append(test2)

    result = test.model_dump()

    expected = {
        "name": "test",
        "number": 1,
        "test2": [
            {
                "names": ["test2"],
                "number": 2,
            },
            {
                "names": ["test2"],
                "number": 2,
            },
        ],
        "ontology": "https://www.evidenceontology.org/term/",
    }

    print("PyDantic XML Test successful")
