// The go edition's driver: one json request per line on stdin, one json response per line on stdout.
package main

import (
	"bufio"
	"encoding/json"
	"math"
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
	in.Buffer(nil, math.MaxInt)       // take a line of any length, as the other three drivers do, rather than stopping at the scanner's default of 64 KB
	out := json.NewEncoder(os.Stdout) // Encode writes a trailing newline, one response per line
	for in.Scan() {
		var r request
		if err := json.Unmarshal(in.Bytes(), &r); err != nil {
			panic(err) // outside the top gate: a line the driver can't read is a bug in the harness, not a call to refuse, so it stops loudly
		}
		response := map[string]any{"id": r.ID}
		if result, err := dispatch.Call(r.Call, r.Args); err != nil {
			response["error"] = err.Error() // the top gate: report anything that escaped as the response, and keep serving
		} else {
			response["result"] = result
		}
		if err := out.Encode(response); err != nil {
			panic(err) // stdout is gone, so there is no one left to answer
		}
	}
	if err := in.Err(); err != nil {
		panic(err) // stdin failed rather than closed, so stop loudly instead of as if the harness were done
	}
}
