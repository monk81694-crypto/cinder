// Usage: cd tools/release && go run . -- Cinder release readiness check; exit 0 = LAUNCH READY, 1 = BLOCKED.
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"time"
)

var requiredFiles = []string{
	"assets/logo.svg",
	"assets/mark.svg",
	"assets/icon.ico",
	"assets/icon.png",
	"assets/logo-512.png",
	"vendor/skinview3d.bundle.js",
	"index.html",
	"styles.css",
	"main.js",
	"preload.js",
	"renderer.js",
	"tests/smoke.js",
}

var semverRe = regexp.MustCompile(`^\d+\.\d+\.\d+([-+][0-9A-Za-z.-]+)?$`)

func main() {
	os.Exit(run())
}

func run() int {
	allOk := true
	root := findRepoRoot()
	fmt.Println("Cinder release check")
	fmt.Printf("repo root: %s\n", root)

	// 1. package.json version field exists and is semver-like.
	version := ""
	pkgPath := filepath.Join(root, "package.json")
	pkgData, err := os.ReadFile(pkgPath)
	if err != nil {
		fmt.Printf("[1/4] package.json version: ERROR cannot read %s: %v\n", pkgPath, err)
		allOk = false
	} else {
		var pkg struct {
			Version string `json:"version"`
		}
		if err := json.Unmarshal(pkgData, &pkg); err != nil {
			fmt.Printf("[1/4] package.json version: ERROR invalid JSON: %v\n", err)
			allOk = false
		} else if strings.TrimSpace(pkg.Version) == "" {
			fmt.Println("[1/4] package.json version: MISSING (no \"version\" field)")
			allOk = false
		} else {
			version = strings.TrimSpace(pkg.Version)
			fmt.Printf("[1/4] package.json version: %s\n", version)
			if !isSemverLike(version) {
				fmt.Printf("  semver check: INVALID (%q is not semver-like, want X.Y.Z)\n", version)
				allOk = false
			} else {
				fmt.Println("  semver check: valid")
			}
		}
	}

	// 2. Version string appears in index.html pills and README.md (report only, do not edit).
	fmt.Printf("[2/4] version references (expect %q):\n", version)
	if version == "" {
		fmt.Println("  index.html tb-ver pill: MISSING (no version to check)")
		fmt.Println("  index.html credit-ver: MISSING (no version to check)")
		fmt.Println("  README.md: MISSING (no version to check)")
		allOk = false
	} else {
		htmlPath := filepath.Join(root, "index.html")
		htmlData, err := os.ReadFile(htmlPath)
		if err != nil {
			fmt.Printf("  index.html tb-ver pill: MISSING (cannot read index.html: %v)\n", err)
			fmt.Printf("  index.html credit-ver: MISSING (cannot read index.html: %v)\n", err)
			allOk = false
		} else {
			html := string(htmlData)
			if inner, found := pillInner(html, "tb-ver"); !found {
				fmt.Println("  index.html tb-ver pill: MISSING (no element with class tb-ver found)")
				allOk = false
			} else if containsVersion(inner, version) {
				fmt.Printf("  index.html tb-ver pill: FOUND (%q contains %q)\n", truncate(inner, 60), version)
			} else {
				fmt.Printf("  index.html tb-ver pill: MISSING (pill text %q does not contain %q)\n", truncate(inner, 60), version)
				allOk = false
			}
			if inner, found := pillInner(html, "credit-ver"); !found {
				fmt.Println("  index.html credit-ver: MISSING (no element with class credit-ver found)")
				allOk = false
			} else if containsVersion(inner, version) {
				fmt.Printf("  index.html credit-ver: FOUND (%q contains %q)\n", truncate(inner, 60), version)
			} else {
				fmt.Printf("  index.html credit-ver: MISSING (credit text %q does not contain %q)\n", truncate(inner, 60), version)
				allOk = false
			}
		}
		readmePath := filepath.Join(root, "README.md")
		readmeData, err := os.ReadFile(readmePath)
		if err != nil {
			fmt.Printf("  README.md: MISSING (cannot read README.md: %v)\n", err)
			allOk = false
		} else if containsVersion(string(readmeData), version) {
			fmt.Printf("  README.md: FOUND (contains %q)\n", version)
		} else {
			fmt.Printf("  README.md: MISSING (does not contain %q)\n", version)
			allOk = false
		}
	}

	// 3. Required files exist.
	fmt.Println("[3/4] required files:")
	for _, rel := range requiredFiles {
		full := filepath.Join(root, rel)
		info, err := os.Stat(full)
		if err != nil {
			fmt.Printf("  %s: MISSING (%v)\n", rel, err)
			allOk = false
		} else if info.IsDir() {
			fmt.Printf("  %s: MISSING (is a directory)\n", rel)
			allOk = false
		} else {
			fmt.Printf("  %s: OK\n", rel)
		}
	}

	// 4. Run npm test in repo root with 3 min timeout.
	fmt.Println("[4/4] npm test (3m timeout):")
	pass, detail := runNpmTest(root)
	fmt.Print(detail)
	if !pass {
		allOk = false
	}

	// 5. Final summary; exit 0 only if everything passes.
	fmt.Println("----")
	if allOk {
		fmt.Printf("RESULT: LAUNCH READY (v%s - all checks passed)\n", version)
		return 0
	}
	fmt.Println("RESULT: BLOCKED - one or more checks failed (see MISSING/FAIL above)")
	return 1
}

func findRepoRoot() string {
	cwd, err := os.Getwd()
	if err != nil {
		return filepath.Join(".", "..", "..")
	}
	dir := cwd
	for i := 0; i < 6; i++ {
		if _, err := os.Stat(filepath.Join(dir, "package.json")); err == nil {
			return dir
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	return filepath.Join(cwd, "..", "..")
}

func isSemverLike(v string) bool {
	s := strings.TrimSpace(v)
	s = strings.TrimPrefix(s, "v")
	return semverRe.MatchString(s)
}

func containsVersion(haystack, version string) bool {
	raw := strings.TrimSpace(version)
	if raw == "" {
		return false
	}
	bare := strings.TrimPrefix(raw, "v")
	candidates := []string{raw, bare, "v" + bare}
	seen := map[string]bool{}
	for _, c := range candidates {
		if c == "" || seen[c] {
			continue
		}
		seen[c] = true
		if strings.Contains(haystack, c) {
			return true
		}
	}
	return false
}

func pillInner(html, class string) (string, bool) {
	pattern := `class=["'][^"']*` + regexp.QuoteMeta(class) + `[^"']*["'][^>]*>([^<]*)<`
	re := regexp.MustCompile(pattern)
	m := re.FindStringSubmatch(html)
	if m == nil {
		return "", false
	}
	return strings.TrimSpace(m[1]), true
}

func truncate(s string, n int) string {
	s = strings.Join(strings.Fields(s), " ")
	if len(s) <= n {
		return s
	}
	return s[:n] + "..."
}

func tailLines(s string, n int) string {
	s = strings.TrimRight(s, "\r\n")
	if s == "" {
		return "(no output)"
	}
	lines := strings.Split(s, "\n")
	if len(lines) <= n {
		return strings.Join(lines, "\n")
	}
	return strings.Join(lines[len(lines)-n:], "\n")
}

func runNpmTest(root string) (bool, string) {
	var b strings.Builder
	npmBin := "npm"
	if _, err := exec.LookPath(npmBin); err != nil {
		if _, err2 := exec.LookPath("npm.cmd"); err2 == nil {
			npmBin = "npm.cmd"
		} else {
			fmt.Fprintf(&b, "  npm test: FAIL (npm not found in PATH: %v)\n", err)
			return false, b.String()
		}
	}
	if _, err := os.Stat(filepath.Join(root, "package.json")); err != nil {
		fmt.Fprintf(&b, "  npm test: FAIL (cannot find package.json in %s: %v)\n", root, err)
		return false, b.String()
	}
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Minute)
	defer cancel()
	cmd := exec.CommandContext(ctx, npmBin, "test")
	cmd.Dir = root
	out, err := cmd.CombinedOutput()
	tail := tailLines(string(out), 20)
	if ctx.Err() == context.DeadlineExceeded {
		fmt.Fprintf(&b, "  npm test: FAIL (timeout after 3m)\n")
		fmt.Fprintf(&b, "  --- output tail (last 20 lines) ---\n%s\n  --- end tail ---\n", tail)
		return false, b.String()
	}
	if err != nil {
		fmt.Fprintf(&b, "  npm test: FAIL (%v)\n", err)
		fmt.Fprintf(&b, "  --- output tail (last 20 lines) ---\n%s\n  --- end tail ---\n", tail)
		return false, b.String()
	}
	fmt.Fprintf(&b, "  npm test: PASS\n")
	fmt.Fprintf(&b, "  --- output tail (last 20 lines) ---\n%s\n  --- end tail ---\n", tail)
	return true, b.String()
}
