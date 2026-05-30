export type AnecdoctConfigValue = string | number | boolean | AnecdoctConfigValue[] | AnecdoctConfigObject;

export type AnecdoctConfigObject = { [key: string]: AnecdoctConfigValue };

export type AnecdoctOptions = {
  anecdoctPathOverride?: string;
  baseUrl?: string;
  apiKey?: string;
  /**
   * Additional `--config key=value` overrides to pass to the Anecdoct CLI.
   *
   * Provide a JSON object and the SDK will flatten it into dotted paths and
   * serialize values as TOML literals so they are compatible with the CLI's
   * `--config` parsing.
   */
  config?: AnecdoctConfigObject;
  /**
   * Environment variables passed to the Anecdoct CLI process. When provided, the SDK
   * will not inherit variables from `process.env`.
   */
  env?: Record<string, string>;
};
