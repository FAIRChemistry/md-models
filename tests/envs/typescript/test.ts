// We are ignoring because the file will be added once the docker build is done
// @ts-ignore
import { TestSchema, Ontology, Test2Schema } from "./generated";

// Test if the schema is valid
const test = TestSchema.parse({
    name: "test",
    number: 1,
    test2: [
        {
            names: ["name1", "name2"],
            number: 123.45,
        },
    ],
    ontology: Ontology.ECO,
});

if (!test) {
    throw new Error("Test schema is invalid");
}

// Test if the schema is valid
const test2 = Test2Schema.parse({
    names: ["name1", "name2"],
    number: 123.45,
});

if (!test2) {
    throw new Error("Test2 schema is invalid");
}

console.log(test);
console.log(test2);
