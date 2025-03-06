// Package model contains Go struct definitions with JSON serialization.
//
// WARNING: This is an auto-generated file.
// Do not edit directly - any changes will be overwritten.

package model

import (
    "gorm.io/gorm"
	"encoding/json"
	"fmt"
)

//
// Type definitions
//

// TestGorm
//
// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
// incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
// nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
type TestGorm struct {
    gorm.Model
    Name string `json:"name" xml:"name,attr" `
    Number float64 `json:"number,omitempty" xml:"number,attr,omitempty" `
    Test2Multiple []Test2 `json:"test2_multiple,omitempty" xml:"test2_multiple,omitempty" gorm:"many2many:testgorm_test2_multiple;"`
    Test2ID int
    Test2Single Test2 `json:"test2_single,omitempty" xml:"test2_single,omitempty" `
    Ontology Ontology `json:"ontology,omitempty" xml:"ontology,omitempty" `
}

// Test2
type Test2 struct {
    gorm.Model
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