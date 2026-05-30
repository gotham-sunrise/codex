import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, beforeEach } from "@jest/globals";

const originalAnecdoctHome = process.env.ANECDOCT_HOME;
let currentAnecdoctHome: string | undefined;

beforeEach(async () => {
  currentAnecdoctHome = await fs.mkdtemp(path.join(os.tmpdir(), "anecdoct-sdk-test-"));
  process.env.ANECDOCT_HOME = currentAnecdoctHome;
});

afterEach(async () => {
  const anecdoctHomeToDelete = currentAnecdoctHome;
  currentAnecdoctHome = undefined;

  if (originalAnecdoctHome === undefined) {
    delete process.env.ANECDOCT_HOME;
  } else {
    process.env.ANECDOCT_HOME = originalAnecdoctHome;
  }

  if (anecdoctHomeToDelete) {
    await fs.rm(anecdoctHomeToDelete, { recursive: true, force: true });
  }
});
