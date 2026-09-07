// Command build compiles the Rust core with cargo and copies the shared library into
// internal/native/lib/<goos>_<goarch>/ so that it is embedded into Go binaries.
//
//	go run ./tools/build            # release build for the host platform
//	go run ./tools/build -debug     # debug build
//	go run ./tools/build -target x86_64-pc-windows-gnu -goos windows -goarch amd64
package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
)

func main() {
	debug := flag.Bool("debug", false, "build the debug profile instead of release")
	target := flag.String("target", "", "cargo --target triple for cross builds")
	goos := flag.String("goos", runtime.GOOS, "GOOS of the embed directory to populate")
	goarch := flag.String("goarch", runtime.GOARCH, "GOARCH of the embed directory to populate")
	flag.Parse()

	root, err := findRoot()
	if err != nil {
		fail(err)
	}
	args := []string{"build", "--manifest-path", filepath.Join(root, "core", "Cargo.toml")}
	profile := "debug"
	if !*debug {
		args = append(args, "--release")
		profile = "release"
	}
	if *target != "" {
		args = append(args, "--target", *target)
	}
	cmd := exec.Command("cargo", args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Dir = root
	fmt.Println("$ cargo", joinArgs(args))
	if err := cmd.Run(); err != nil {
		fail(fmt.Errorf("cargo build failed: %w (is Rust installed? https://rustup.rs)", err))
	}

	name := libName(*goos)
	outDir := filepath.Join(root, "target")
	if *target != "" {
		outDir = filepath.Join(outDir, *target)
	}
	src := filepath.Join(outDir, profile, name)
	dst := filepath.Join(root, "internal", "native", "lib", *goos+"_"+*goarch, name)
	if err := copyFile(src, dst); err != nil {
		fail(err)
	}
	fmt.Printf("copied %s\n    -> %s\n", src, dst)
}

func libName(goos string) string {
	switch goos {
	case "windows":
		return "patina_core.dll"
	case "darwin":
		return "libpatina_core.dylib"
	default:
		return "libpatina_core.so"
	}
}

// findRoot walks up from the working directory to the directory containing go.mod.
func findRoot() (string, error) {
	dir, err := os.Getwd()
	if err != nil {
		return "", err
	}
	for {
		if _, err := os.Stat(filepath.Join(dir, "go.mod")); err == nil {
			if _, err := os.Stat(filepath.Join(dir, "core", "Cargo.toml")); err == nil {
				return dir, nil
			}
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return "", fmt.Errorf("run this from inside the patina repository (go.mod and core/Cargo.toml not found)")
		}
		dir = parent
	}
}

func copyFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return fmt.Errorf("cannot open build output %s: %w", src, err)
	}
	defer in.Close()
	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return err
	}
	tmp := dst + ".tmp"
	out, err := os.Create(tmp)
	if err != nil {
		return err
	}
	if _, err := io.Copy(out, in); err != nil {
		out.Close()
		return err
	}
	if err := out.Close(); err != nil {
		return err
	}
	return os.Rename(tmp, dst)
}

func joinArgs(args []string) string {
	s := ""
	for i, a := range args {
		if i > 0 {
			s += " "
		}
		s += a
	}
	return s
}

func fail(err error) {
	fmt.Fprintln(os.Stderr, "error:", err)
	os.Exit(1)
}
