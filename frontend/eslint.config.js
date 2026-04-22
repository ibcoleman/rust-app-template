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
    },
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        {argsIgnorePattern: "^_"},
      ],
      "@typescript-eslint/no-explicit-any": "error",
    },
  },
];
