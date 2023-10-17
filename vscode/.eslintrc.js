const OFF = "off";
const WARN = "warn";
const ERROR = "error";

module.exports = {
    root: true,
    env: {
        es2021: true,
        node: true,
    },
    extends: ["eslint:recommended", "plugin:@typescript-eslint/recommended", "plugin:prettier/recommended"],
    parser: "@typescript-eslint/parser",
    parserOptions: {
        ecmaVersion: 12,
        sourceType: "module",
    },
    plugins: ["@typescript-eslint", "simple-import-sort"],
    rules: {
        "linebreak-style": [ERROR, "unix"],
        "@typescript-eslint/explicit-module-boundary-types": OFF,
        "eol-last": WARN,
        "simple-import-sort/imports": [
            ERROR,
            {
                groups: [
                    ["^\\u0000.*(?<!\\.s?css)$"], // Side effect imports (but not css)
                    ["^@?\\w"], // node builtins and external packages
                    ["^(?!(\\.|@\\/))"], // anything that's not a relative import
                    ["^@\\/"], // absolute imports
                    ["^\\."], // relative imports
                ],
            },
        ],
        "simple-import-sort/exports": ERROR,
        "object-curly-spacing": [ERROR, "always"],
        "@typescript-eslint/member-delimiter-style": ERROR,
        "@typescript-eslint/no-non-null-assertion": OFF,
        "@typescript-eslint/no-namespace": OFF,
        "@typescript-eslint/no-explicit-any": OFF,
        "prefer-const": [
            ERROR,
            {
                destructuring: "all",
            },
        ],
        "@typescript-eslint/no-empty-interface": OFF,
        "@typescript-eslint/no-empty-function": OFF,
        "no-inner-declarations": OFF,
        "@typescript-eslint/no-non-null-asserted-optional-chain": OFF,
        "no-constant-condition": OFF,
        "no-async-promise-executor": OFF,
    },
    overrides: [
        {
            files: [".*.js", "*.json"],
            rules: {
                "@typescript-eslint/naming-convention": OFF,
            },
        },
    ],
};
