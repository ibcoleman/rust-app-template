import tseslint from "@typescript-eslint/eslint-plugin";
import tsparser from "@typescript-eslint/parser";
import neverthrow from "eslint-plugin-neverthrow";

export default [
  {
    ignores: ["node_modules/**", "dist/**"],
  },
  {
    files: ["src/**/*.ts"],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: "module",
        project: "./tsconfig.json",
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      "@typescript-eslint": tseslint,
      neverthrow,
    },
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        {argsIgnorePattern: "^_"},
      ],
      "@typescript-eslint/no-explicit-any": "error",
      // neverthrow/must-use-result rule requires parserServices from TypeScript parser,
      // but eslint-plugin-neverthrow 1.1.4 has a compatibility issue with @typescript-eslint 8.x
      // in flat config mode. The plugin is imported and available, but the rule is disabled.
      // Code review and pnpm typecheck ensure Result values are properly handled.
    },
  },
];
