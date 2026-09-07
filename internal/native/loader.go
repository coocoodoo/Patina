package native

import (
	"crypto/sha256"
	"embed"
	"encoding/hex"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
)

// Prebuilt cores are embedded from lib/<goos>_<goarch>/ when present, so `go build` produces a
// self-contained executable. `go run ./tools/build` populates that directory.
//
//go:embed lib
var embedded embed.FS

// LibName is the platform file name of the core library.
func LibName() string {
	switch runtime.GOOS {
	case "windows":
		return "patina_core.dll"
	case "darwin":
		return "libpatina_core.dylib"
	default:
		return "libpatina_core.so"
	}
}

func fileExists(p string) bool {
	fi, err := os.Stat(p)
	return err == nil && !fi.IsDir()
}

// locate finds the shared library, in this order: the PATINA_LIBRARY environment variable,
// an embedded copy, next to the executable, and finally cargo output directories above the
// working directory (handy while developing).
func locate() (string, error) {
	if p := os.Getenv("PATINA_LIBRARY"); p != "" {
		if fileExists(p) {
			return p, nil
		}
		return "", fmt.Errorf("patina: PATINA_LIBRARY=%q does not exist", p)
	}
	if p, ok := extractEmbedded(); ok {
		return p, nil
	}
	name := LibName()
	var candidates []string
	if exe, err := os.Executable(); err == nil {
		candidates = append(candidates, filepath.Join(filepath.Dir(exe), name))
	}
	if cwd, err := os.Getwd(); err == nil {
		dir := cwd
		for i := 0; i < 8; i++ {
			candidates = append(candidates,
				filepath.Join(dir, name),
				filepath.Join(dir, "target", "release", name),
				filepath.Join(dir, "target", "debug", name),
				filepath.Join(dir, "core", "target", "release", name),
				filepath.Join(dir, "core", "target", "debug", name),
			)
			parent := filepath.Dir(dir)
			if parent == dir {
				break
			}
			dir = parent
		}
	}
	for _, c := range candidates {
		if fileExists(c) {
			return c, nil
		}
	}
	return "", fmt.Errorf("patina: native library %s not found.\n"+
		"Build it with `go run ./tools/build` (needs cargo) or point PATINA_LIBRARY at it.\n"+
		"Looked in:\n  %s", name, strings.Join(candidates, "\n  "))
}

// extractEmbedded writes the embedded core (if any) to the user cache dir and returns its path.
func extractEmbedded() (string, bool) {
	rel := path.Join("lib", runtime.GOOS+"_"+runtime.GOARCH, LibName())
	data, err := embedded.ReadFile(rel)
	if err != nil || len(data) == 0 {
		return "", false
	}
	sum := sha256.Sum256(data)
	base, err := os.UserCacheDir()
	if err != nil || base == "" {
		base = os.TempDir()
	}
	dst := filepath.Join(base, "patina", hex.EncodeToString(sum[:10]), LibName())
	if fi, err := os.Stat(dst); err == nil && fi.Size() == int64(len(data)) {
		return dst, true
	}
	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return "", false
	}
	tmp := dst + ".tmp" + strconv.Itoa(os.Getpid())
	if err := os.WriteFile(tmp, data, 0o755); err != nil {
		return "", false
	}
	if err := os.Rename(tmp, dst); err != nil {
		_ = os.Remove(tmp)
		if !fileExists(dst) {
			return "", false
		}
	}
	return dst, true
}
