import neverthrow from "eslint-plugin-neverthrow";

export default [
  {
    files: ["src/**/*.ts"],
    plugins: { neverthrow },
    rules: {
      "neverthrow/must-use-result": "error",
    },
  },
];
