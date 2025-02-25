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

// Test
//
// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
// incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
// nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
type Test struct {
        ID *uint `json:"id,omitempty" xml:"id,attr,omitempty" gorm:"primaryKey,autoIncrement"`
        Name string `json:"name" xml:"name,attr" `
        Number TestNumberType `json:"number,omitempty" xml:"number,attr,omitempty" `
        Test2 []Test2 `json:"test2,omitempty" xml:"SomeTest2,omitempty" gorm:"many2many:test_test2;"`
        Ontology Ontology `json:"ontology,omitempty" xml:"ontology,omitempty" `
}

// Test2
type Test2 struct {
        ID *uint `json:"id,omitempty" xml:"id,attr,omitempty" gorm:"primaryKey,autoIncrement"`
        Names []string `json:"names,omitempty" xml:"name,omitempty" `
        Number float64 `json:"number,omitempty" xml:"number,attr,omitempty" `
}

//
// Enum definitions
//
type Ontology string

const (
    ECO Ontology = "https://www.evidenceontology.org/term/"
    GO Ontology = "https://amigo.geneontology.org/amigo/term/"
    SIO Ontology = "http://semanticscience.org/resource/"
)

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
    if t.Float != 0 {
        return json.Marshal(t.Float)
    }
    if t.String != "" {
        return json.Marshal(t.String)
    }
    return []byte("null"), nil
}