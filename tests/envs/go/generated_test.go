package model

import (
	"encoding/json"
	"testing"
)

func TestTestStructSerialization(t *testing.T) {
	// Test basic struct serialization
	test := Test{
		Name: "test1",
		Number: TestNumberType{
			Float: 42.0,
		},
		Test2: []Test2{
			{
				Names:  []string{"name1", "name2"},
				Number: 123.45,
			},
		},
		Ontology: ECO,
	}

	// Test marshaling
	data, err := json.Marshal(test)
	if err != nil {
		t.Errorf("Failed to marshal Test struct: %v", err)
	}

	// Test unmarshaling
	var decoded Test
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Errorf("Failed to unmarshal Test struct: %v", err)
	}

	// Verify fields
	if decoded.Name != test.Name {
		t.Errorf("Name mismatch: got %v, want %v", decoded.Name, test.Name)
	}
	if decoded.Number.Float != test.Number.Float {
		t.Errorf("Number mismatch: got %v, want %v", decoded.Number.Float, test.Number.Float)
	}
}

func TestTestNumberTypeUnion(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		wantNum float64
		wantStr string
		wantErr bool
	}{
		{
			name:    "float value",
			input:   "42.5",
			wantNum: 42.5,
		},
		{
			name:    "string value",
			input:   `"test-string"`,
			wantStr: "test-string",
		},
		{
			name:    "invalid value",
			input:   "true",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var num TestNumberType
			err := json.Unmarshal([]byte(tt.input), &num)

			if tt.wantErr {
				if err == nil {
					t.Error("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if num.Float != tt.wantNum {
				t.Errorf("Float mismatch: got %v, want %v", num.Float, tt.wantNum)
			}
			if num.String != tt.wantStr {
				t.Errorf("String mismatch: got %v, want %v", num.String, tt.wantStr)
			}
		})
	}
}
