import type { Lintkit } from "../src/plugins/registry.js";

export function register(lintkit: Lintkit) {
  lintkit.addRule({
    name: "max-lines",
    check: (_file, text) => (text.split("\n").length > 300 ? ["over 300 lines"] : []),
  });
}
