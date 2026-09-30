import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Options } from "@wdio/types";

const root = path.dirname(fileURLToPath(import.meta.url));

export const config: Options.Testrunner = {
  runner: "local",
  specs: ["./e2e/specs/**/*.ts"],
  maxInstances: 1,
  framework: "mocha",
  reporters: ["spec"],
  outputDir: path.join(root, "logs"),
  services: [
    [
      "@wdio/tauri-service",
      {
        appBinaryPath: path.join(
          process.env.CARGO_TARGET_DIR ?? path.join(root, "target"),
          "debug/lumin",
        ),
        driverProvider: "external",
        autoInstallTauriDriver: true,
      },
    ],
  ],
  capabilities: {
    teacher: {
      capabilities: {
        browserName: "tauri",
        "tauri:options": {
          application: path.join(
            process.env.CARGO_TARGET_DIR ?? path.join(root, "target"),
            "debug/lumin",
          ),
        },
      },
    },
    student: {
      capabilities: {
        browserName: "tauri",
        "tauri:options": {
          application: path.join(
            process.env.CARGO_TARGET_DIR ?? path.join(root, "target"),
            "debug/lumin",
          ),
        },
      },
    },
  },
  mochaOpts: { timeout: 60_000 },
  waitforTimeout: 10_000,
};
