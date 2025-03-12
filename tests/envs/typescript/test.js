"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var generated_1 = require("./generated");
// Test if the schema is valid
var test = generated_1.TestSchema.parse({
    name: "test",
    number: 1,
    test2: [
        {
            names: ["name1", "name2"],
            number: 123.45,
        },
    ],
    ontology: generated_1.Ontology.ECO,
});
if (!test) {
    throw new Error("Test schema is invalid");
}
// Test if the schema is valid
var test2 = generated_1.Test2Schema.parse({
    names: ["name1", "name2"],
    number: 123.45,
});
if (!test2) {
    throw new Error("Test2 schema is invalid");
}
console.log(test);
console.log(test2);
