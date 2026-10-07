import type { Lintkit } from "../src/plugins/registry.js";

export function register(lintkit: Lintkit) {
  lintkit.addRule({
    name: "no-console",
    check: (_file, text) => (text.includes("console.log") ? ["console.log left in"] : []),
    fix: (text) => text.replaceAll(/^\s*console\.log\(.*\);?\n/gm, ""),
  });
}
