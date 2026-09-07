# Embedded native cores

`go run ./tools/build` compiles `core/` with cargo and copies the resulting shared library into
`<goos>_<goarch>/` next to this file, for example `windows_amd64/patina_core.dll`. Anything placed
here is embedded into Go binaries with `//go:embed`, so a plain `go build` yields a self-contained
executable.

If no core is embedded, the package falls back to `PATINA_LIBRARY`, the executable's directory,
and cargo's `target/` directories above the working directory.
