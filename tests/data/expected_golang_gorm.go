// Package model contains Go struct definitions with JSON serialization.
//
// WARNING: This is an auto-generated file.
// Do not edit directly - any changes will be overwritten.

package model

//
// Type definitions
//

// TestGorm
//
// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
// incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
// nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
type TestGorm struct {
    Id int64 `json:"-" gorm:"primaryKey;autoIncrement"`
    Name string `json:"name" `
    Number float64 `json:"number,omitempty" `
    Test2Multiple []Test2 `json:"test2_multiple,omitempty" gorm:"many2many:testgorm_test2_multiple;"`
    Test2SingleID int64 `json:"-"`
    Test2Single Test2 `json:"test2_single,omitempty" gorm:"foreignKey:Test2SingleID;"`
    Ontology Ontology `json:"ontology,omitempty" `
}

// Test2
type Test2 struct {
    Id int64 `json:"-" gorm:"primaryKey;autoIncrement"`
    Names []string `json:"names,omitempty" gorm:"serializer:json;"`
    Number float64 `json:"number,omitempty" `
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