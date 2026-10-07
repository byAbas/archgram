import { readdirSync } from "node:fs";

export type Rule = { name: string; check(file: string, text: string): string[]; fix?(text: string): string };

// What lintkit hands each plugin: the one call a plugin makes back.
export type Lintkit = { addRule(rule: Rule): void };

export async function loadRules(): Promise<Rule[]> {
  const rules: Rule[] = [];
  const lintkit: Lintkit = { addRule: (rule) => rules.push(rule) };
  for (const file of readdirSync("plugins")) {
    const plugin = await import(`../../plugins/${file}`);
    plugin.register(lintkit);
  }
  return rules;
}
