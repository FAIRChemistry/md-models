// Package model contains Go struct definitions with JSON serialization.
//
// WARNING: This is an auto-generated file.
// Do not edit directly - any changes will be overwritten.

package model

import (
    "encoding/json"
    "fmt"
)

//
// Type definitions
//

// Test Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
// eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim
// ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
// aliquip ex ea commodo consequat.
type Test struct {
        // The name of the test. This is a unique identifier that helps track
        // individual test cases across the system. It should be
        // descriptive and follow the standard naming conventions.
        Name string `json:"name"`
        Number TestNumberType `json:"number,omitempty"`
        Test2 []Test2 `json:"test2,omitempty"`
        Ontology Ontology `json:"ontology,omitempty"`
}

type Test2 struct {
        Names []string `json:"names,omitempty"`
        Number float64 `json:"number,omitempty"`
}

//
// Enum definitions
//
type Ontology string

const (
    OntologyECO Ontology = "https://www.evidenceontology.org/term/"
    OntologyGO Ontology = "https://amigo.geneontology.org/amigo/term/"
    OntologySIO Ontology = "http://semanticscience.org/resource/"
)

//
// Type definitions for attributes with multiple types
//

// TestNumberType represents a union type that can hold any of the following types:
// - float
// - string
type TestNumberType struct {
    Float float64
    String string
}

// UnmarshalJSON implements custom JSON unmarshaling for TestNumberType
func (t *TestNumberType) UnmarshalJSON(data []byte) error {
    // Reset existing values
    t.Float = 0
    t.String = ""
    var floatValue float64
    if err := json.Unmarshal(data, &floatValue); err == nil {
        t.Float = floatValue
        return nil
    }
    var stringValue string
    if err := json.Unmarshal(data, &stringValue); err == nil {
        t.String = stringValue
        return nil
    }
    return fmt.Errorf("TestNumberType: data is neither float, string")
}

// MarshalJSON implements custom JSON marshaling for TestNumberType
func (t TestNumberType) MarshalJSON() ([]byte, error) {
    if t.Float != nil {
        return json.Marshal(*t.Float)
    }
    if t.String != nil {
        return json.Marshal(*t.String)
    }
    return []byte("null"), nil
}