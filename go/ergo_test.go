package ergo_test

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"

	"github.com/zootella/ergo/go/internal/dispatch"
)

// vector is one case from the shared test vectors
type vector struct {
	Function string `json:"function"`
	Input    []any  `json:"input"`
	Output   any    `json:"output"`
	Throws   bool   `json:"throws"`
	Note     string `json:"note"`
}

// TestVectors runs every case in the shared test vectors against this edition.
func TestVectors(t *testing.T) {
	files, err := filepath.Glob("../tests/*.json") // the shared vectors beside this edition
	if err != nil || len(files) == 0 {
		t.Fatal("no shared vectors found beside this module")
	}
	for _, file := range files {
		data, err := os.ReadFile(file)
		if err != nil {
			t.Fatal(err)
		}
		var cases []vector
		if err := json.Unmarshal(data, &cases); err != nil {
			t.Fatal(err)
		}
		for _, c := range cases {
			t.Run(fmt.Sprintf("%s(%v) %s", c.Function, c.Input, c.Note), func(t *testing.T) {
				got, err := dispatch.Call(c.Function, c.Input)
				switch {
				case c.Throws && err == nil:
					t.Errorf("expected a panic, got %v", got)
				case c.Throws && strings.HasPrefix(err.Error(), "unknown function"):
					t.Errorf("expected a panic, got %v; an unknown function refuses everything, so it can't pass a throws vector", err)
				case !c.Throws && err != nil:
					t.Errorf("unexpected error: %v", err)
				case !c.Throws && !reflect.DeepEqual(viaJSON(got), c.Output):
					t.Errorf("expected %v, got %v", c.Output, got)
				}
			})
		}
	}
}

// viaJSON takes a result through json and back, so an int compares equal to the float64 that json gave the expected output
func viaJSON(v any) any {
	b, _ := json.Marshal(v)
	var out any
	_ = json.Unmarshal(b, &out)
	return out
}
