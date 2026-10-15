// The go edition's driver: one json request per line on stdin, one json response per line on stdout.
package main

import (
	"bufio"
	"encoding/json"
	"os"

	"github.com/zootella/ergo/go/internal/dispatch"
)

// request is one line from the harness
type request struct {
	ID   any    `json:"id"`
	Call string `json:"call"`
	Args []any  `json:"args"`
}

func main() {
	in := bufio.NewScanner(os.Stdin)
	out := json.NewEncoder(os.Stdout) // Encode writes a trailing newline, one response per line
	for in.Scan() {
		var r request
		response := map[string]any{}
		if err := json.Unmarshal(in.Bytes(), &r); err != nil {
			response["error"] = err.Error()
		} else {
			response["id"] = r.ID
			if result, err := dispatch.Call(r.Call, r.Args); err != nil {
				response["error"] = err.Error() // the top gate: report anything that escaped as the response, and keep serving
			} else {
				response["result"] = result
			}
		}
		if err := out.Encode(response); err != nil {
			panic(err) // stdout is gone, so there is no one left to answer
		}
	}
}
