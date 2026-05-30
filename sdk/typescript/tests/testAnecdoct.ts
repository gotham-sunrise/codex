import path from "node:path";

import { Anecdoct } from "../src/anecdoct";
import type { AnecdoctConfigObject } from "../src/anecdoctOptions";

export const anecdoctExecPath =
  process.env.ANECDOCT_EXEC_PATH ??
  path.join(process.cwd(), "..", "..", "anecdoct-rs", "target", "debug", "anecdoct");

type CreateTestClientOptions = {
  apiKey?: string;
  baseUrl?: string;
  config?: AnecdoctConfigObject;
  env?: Record<string, string>;
  inheritEnv?: boolean;
};

export type TestClient = {
  cleanup: () => void;
  client: Anecdoct;
};

export function createMockClient(url: string): TestClient {
  return createTestClient({
    config: {
      model_provider: "mock",
      model_providers: {
        mock: {
          name: "Mock provider for test",
          base_url: url,
          wire_api: "responses",
          supports_websockets: false,
        },
      },
    },
  });
}

export function createTestClient(options: CreateTestClientOptions = {}): TestClient {
  const env =
    options.inheritEnv === false ? { ...options.env } : { ...getCurrentEnv(), ...options.env };

  return {
    cleanup: () => {},
    client: new Anecdoct({
      anecdoctPathOverride: anecdoctExecPath,
      baseUrl: options.baseUrl,
      apiKey: options.apiKey,
      config: mergeTestConfig(options.baseUrl, options.config),
      env,
    }),
  };
}

function mergeTestConfig(
  baseUrl: string | undefined,
  config: AnecdoctConfigObject | undefined,
): AnecdoctConfigObject | undefined {
  const mergedConfig: AnecdoctConfigObject | undefined =
    !baseUrl || hasExplicitProviderConfig(config)
      ? config
      : {
          ...config,
          // Built-in providers are merged before user config, so tests need a
          // custom provider entry to force SSE against the local mock server.
          model_provider: "mock",
          model_providers: {
            mock: {
              name: "Mock provider for test",
              base_url: baseUrl,
              wire_api: "responses",
              supports_websockets: false,
            },
          },
        };
  const featureOverrides = mergedConfig?.features;

  return {
    ...mergedConfig,
    // Disable plugins in SDK integration tests so background curated-plugin
    // sync does not race temp ANECDOCT_HOME cleanup.
    features:
      featureOverrides && typeof featureOverrides === "object" && !Array.isArray(featureOverrides)
        ? { ...featureOverrides, plugins: false }
        : { plugins: false },
  };
}

function hasExplicitProviderConfig(config: AnecdoctConfigObject | undefined): boolean {
  return config?.model_provider !== undefined || config?.model_providers !== undefined;
}

function getCurrentEnv(): Record<string, string> {
  const env: Record<string, string> = {};

  for (const [key, value] of Object.entries(process.env)) {
    if (key === "ANECDOCT_INTERNAL_ORIGINATOR_OVERRIDE") {
      continue;
    }
    if (value !== undefined) {
      env[key] = value;
    }
  }

  return env;
}
