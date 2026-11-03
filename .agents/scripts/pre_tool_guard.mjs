import fs from "node:fs";

let raw = "";
try {
  raw = fs.readFileSync(0, "utf8");
} catch {
  process.stdout.write(JSON.stringify({ decision: "allow" }));
  process.exit(0);
}

let payload = {};
try {
  payload = JSON.parse(raw || "{}");
} catch {
  process.stdout.write(JSON.stringify({ decision: "allow" }));
  process.exit(0);
}

const cmd = String(payload?.toolCall?.args?.CommandLine || "");

// Ignore git commit / echo / write statements when matching shell commands
const isGitCommitOnly = /^\s*git\s+(add|commit|status|log)\b/i.test(cmd) && !/\bcargo\s+test\b.*;\s*cargo/i.test(cmd);

if (!isGitCommitOnly) {
  // 1. Block actual `cargo test --release` invocations (outside of quotes)
  if (/(?:^|[;&|]\s*)cargo\s+test\s+[^"'\r\n]*--release\b/i.test(cmd)) {
    process.stdout.write(
      JSON.stringify({
        decision: "deny",
        reason:
          "strata-arch3d Cargo.toml sets `panic = \"abort\"` in [profile.release]. Run `cargo test --workspace` in the default dev profile instead of `--release`."
      })
    );
    process.exit(0);
  }

  // 2. Hard block destructive git force-push
  if (/(?:^|[;&|]\s*)git\s+push\s+[^"'\r\n]*(\s--force\b|\s-f\b)/i.test(cmd)) {
    process.stdout.write(
      JSON.stringify({
        decision: "deny",
        reason: "Force push (`git push --force` / `-f`) is prohibited by workspace safety rules."
      })
    );
    process.exit(0);
  }
}

process.stdout.write(JSON.stringify({ decision: "allow" }));
