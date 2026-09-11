#!/usr/bin/env bun
/**
 * Download Gemma ONNX models for Lumin.
 * Usage:
 *   bun run download:model              # default 1B INT4
 *   bun run download:model --variant 1b-int8
 *   bun run download:model --variant 4-e2b-int4
 *   bun run download:model --variant 4-e4b-int4
 *   bun scripts/download_model.ts --variant 1b-int4 --out models
 *
 * Requires HF_TOKEN for gated Gemma models if using source repos.
 * For onnx-community, no token needed for the pre-converted ONNX.
 */

interface VariantFile {
  /** Path inside the Hugging Face repo. */
  path: string;
  /** Filename on disk. External data MUST keep its upstream name: the .onnx
   *  graph references it by the literal name recorded at export time. */
  dest: string;
  /** SHA256 from the HF API (lfs.oid). */
  sha: string;
}

interface VariantSpec {
  repo: string;
  /** Subdirectory under --out, or "" for the root. Mirrors `dir_name` in the
   *  Rust catalog: variants whose external data shares a generic upstream
   *  name need their own directory. */
  dir: string;
  desc: string;
  files: VariantFile[];
}

/**
 * Mirrors `src-tauri/src/inference/catalog.rs`, which is the source of truth.
 * Keep the two in sync: the app looks for exactly these paths.
 */
const VARIANTS: Record<string, VariantSpec> = {
  "1b-int4": {
    repo: "onnx-community/gemma-3-1b-it-ONNX",
    dir: "",
    desc: "Gemma 3 1B INT4 (community ONNX)",
    files: [
      {
        path: "onnx/model_q4.onnx",
        dest: "gemma-3-1b-it-int4.onnx",
        sha: "69686023e5892376e38fcbcdd0c77af432c55b3bcd03aee6d561bd1f04507da0",
      },
      {
        path: "onnx/model_q4.onnx_data",
        dest: "model_q4.onnx_data",
        sha: "c2370070be257a98d50e17d81be13e18304c39e7e6d9d1416f8f883681d2a17b",
      },
      {
        path: "tokenizer.json",
        dest: "tokenizer.json",
        sha: "55da1312bdf1d7d8fe8d9d1b3eed04086261149e6034e0ac3f8c633b67f5aac8",
      },
    ],
  },
  "1b-int8": {
    repo: "onnx-community/gemma-3-1b-it-ONNX",
    dir: "",
    desc: "Gemma 3 1B INT8",
    files: [
      {
        path: "onnx/model_int8.onnx",
        dest: "gemma-3-1b-it-int8.onnx",
        sha: "6d8ddeb9c637d43625df45933ad3a9e2337b8a027ab37a70dc230735ba285f5c",
      },
      {
        path: "tokenizer.json",
        dest: "tokenizer.json",
        sha: "55da1312bdf1d7d8fe8d9d1b3eed04086261149e6034e0ac3f8c633b67f5aac8",
      },
    ],
  },
  "3n-e2b-int4": {
    repo: "onnx-community/gemma-3n-E2B-it-ONNX",
    dir: "gemma-3n-e2b-int4",
    desc: "Gemma 3n E2B INT4 (mobile optimized)",
    files: [
      {
        path: "onnx/decoder_model_merged_q4.onnx",
        dest: "decoder_model_merged_q4.onnx",
        sha: "4fcb3a37937db577756270c504851e9366ffa738ace6c5ee7d345728aa8dcbd0",
      },
      {
        path: "onnx/decoder_model_merged_q4.onnx_data",
        dest: "decoder_model_merged_q4.onnx_data",
        sha: "297a9301058969f1e67e42546a48875b4250f58b10a28249ff08d76e0b5ead57",
      },
      {
        path: "onnx/embed_tokens_q4.onnx",
        dest: "embed_tokens_q4.onnx",
        sha: "54431e64a782ed74220a0af6b4184a6bf59e4b47e0b14ea4e1335810bc0f4f9b",
      },
      {
        path: "onnx/embed_tokens_q4.onnx_data",
        dest: "embed_tokens_q4.onnx_data",
        sha: "a94d38d81555f7143d1f2108433f866aa54cd4b574e3a527e3edacc34c475d70",
      },
      {
        path: "tokenizer.json",
        dest: "tokenizer.json",
        sha: "44cb3d7d545cf895311e004d9a2b2ce823be5eb84c9aa31f73858b607c44c924",
      },
    ],
  },
  "4-e2b-int4": {
    repo: "onnx-community/gemma-4-E2B-it-ONNX",
    dir: "gemma-4-e2b-int4",
    desc: "Gemma 4 E2B INT4",
    files: [
      {
        path: "onnx/decoder_model_merged_q4.onnx",
        dest: "decoder_model_merged_q4.onnx",
        sha: "c6edb929bf342c524728d37efd400285ee71525e8fe64ff996341f78c3e577d2",
      },
      {
        path: "onnx/decoder_model_merged_q4.onnx_data",
        dest: "decoder_model_merged_q4.onnx_data",
        sha: "b879fe4b946c9b9ff6acb60f7c5eda3d2c9c4df8625895feb2d1e269002f0345",
      },
      {
        path: "onnx/embed_tokens_q4.onnx",
        dest: "embed_tokens_q4.onnx",
        sha: "2d8c8a2bcc30e8ded7f636967c2a58a346116583356dd933720b005fc88079c4",
      },
      {
        path: "onnx/embed_tokens_q4.onnx_data",
        dest: "embed_tokens_q4.onnx_data",
        sha: "40fa957d9988b8a0160c8b0eb5c3f781a237627e9f7153f30514a4ffb2e62888",
      },
      {
        path: "tokenizer.json",
        dest: "tokenizer.json",
        sha: "47bd35616c7c782aaca6ccf48c75f3461d5877170984b8836b375107d0a9f566",
      },
    ],
  },
  "4-e4b-int4": {
    repo: "onnx-community/gemma-4-E4B-it-ONNX",
    dir: "gemma-4-e4b-int4",
    desc: "Gemma 4 E4B INT4",
    files: [
      {
        path: "onnx/decoder_model_merged_q4.onnx",
        dest: "decoder_model_merged_q4.onnx",
        sha: "af764e3b468255401694cf9acbdf859e7e2febd8cd9abd1443ab8ab5493d1f99",
      },
      {
        path: "onnx/decoder_model_merged_q4.onnx_data",
        dest: "decoder_model_merged_q4.onnx_data",
        sha: "3f0dda0cd6b575a49c9a8753250256518a2b9a053ed4cc9dddb6e228a3732099",
      },
      // E4B's weights exceed the 2 GiB protobuf limit and are sharded.
      {
        path: "onnx/decoder_model_merged_q4.onnx_data_1",
        dest: "decoder_model_merged_q4.onnx_data_1",
        sha: "3ce62ea330a5549d791d9b7a2a72dcad8cd4f4083fac490b72b1d539f46f07eb",
      },
      {
        path: "onnx/embed_tokens_q4.onnx",
        dest: "embed_tokens_q4.onnx",
        sha: "78f0c7025bc90339807336da699d7dac6c51ddbb4a690aa4c9e68f3e42b2b353",
      },
      {
        path: "onnx/embed_tokens_q4.onnx_data",
        dest: "embed_tokens_q4.onnx_data",
        sha: "67a3100d70c26840c34f8aac29aeec3db31315268cfc53dbc456e2a648849f35",
      },
      {
        path: "onnx/embed_tokens_q4.onnx_data_1",
        dest: "embed_tokens_q4.onnx_data_1",
        sha: "260b86f8e306ea73aeb8f85c0c142e1fef4a5ac1889a2345e35891518d9367d5",
      },
      {
        path: "tokenizer.json",
        dest: "tokenizer.json",
        sha: "47bd35616c7c782aaca6ccf48c75f3461d5877170984b8836b375107d0a9f566",
      },
    ],
  },
};

async function downloadFile(
  url: string,
  dest: string,
  expectedSha?: string,
  token?: string,
) {
  const headers: Record<string, string> = {};
  if (token) headers.Authorization = `Bearer ${token}`;
  const part = `${dest}.part`;
  const IDLE_TIMEOUT_MS = 60_000;
  const controller = new AbortController();
  let idleTimer: ReturnType<typeof setTimeout> | null = null;
  const resetIdleTimer = () => {
    if (idleTimer) clearTimeout(idleTimer);
    idleTimer = setTimeout(
      () =>
        controller.abort(
          new Error(`stalled transfer: no data for ${IDLE_TIMEOUT_MS / 1000}s`),
        ),
      IDLE_TIMEOUT_MS,
    );
  };
  try {
    resetIdleTimer();
    const res = await fetch(url, { headers, signal: controller.signal });
    if (!res.ok || !res.body) {
      throw new Error(
        `Failed to fetch ${url}: ${res.status} ${res.statusText} - ${await res.text().catch(() => "")}`,
      );
    }
    const hasher = new Bun.CryptoHasher("sha256");
    const writer = Bun.file(part).writer();
    let bytes = 0;
    for await (const chunk of res.body) {
      const buf = chunk as Uint8Array;
      resetIdleTimer();
      hasher.update(buf);
      if (idleTimer) clearTimeout(idleTimer);
      await writer.write(buf);
      resetIdleTimer();
      bytes += buf.byteLength;
      process.stdout.write(`\r  ↓ ${(bytes / 1024 / 1024).toFixed(1)} MB`);
    }
    await writer.end();
    process.stdout.write("\n");
    const actualSha = hasher.digest("hex");
    if (expectedSha && actualSha !== expectedSha) {
      throw new Error(
        `SHA256 mismatch for ${dest}: expected ${expectedSha}, got ${actualSha}`,
      );
    }
    await Bun.$`mv ${part} ${dest}`.quiet();
    console.log(
      `  ✓ ${dest} (${(bytes / 1024 / 1024).toFixed(1)} MB, sha256 ✓)`,
    );
  } catch (e) {
    await Bun.$`rm -f ${part}`.quiet();
    throw e;
  } finally {
    if (idleTimer) clearTimeout(idleTimer);
  }
}

async function main() {
  const args = process.argv.slice(2);
  const variantArg =
    args.find((a) => a.startsWith("--variant="))?.split("=")[1] ??
    args[args.indexOf("--variant") + 1];
  const outArg =
    args.find((a) => a.startsWith("--out="))?.split("=")[1] ??
    args[args.indexOf("--out") + 1];
  const variant = variantArg ?? "1b-int4";
  const outDir = outArg ?? "models";

  if (!(variant in VARIANTS)) {
    console.error(
      `Unknown variant ${variant}. Choose: ${Object.keys(VARIANTS).join(", ")}`,
    );
    process.exit(1);
  }

  const cfg = VARIANTS[variant];
  const destDir = cfg.dir ? `${outDir}/${cfg.dir}` : outDir;
  const token = process.env.HF_TOKEN || process.env.HUGGING_FACE_HUB_TOKEN;
  console.log(`\n[lumin] Download ${variant}: ${cfg.desc}`);
  console.log(`  repo: ${cfg.repo}`);
  console.log(`  out:  ${destDir}/`);
  if (!token)
    console.log(
      `  HF_TOKEN not set — gated models may fail (onnx-community usually OK)`,
    );

  await Bun.$`mkdir -p ${destDir}`.quiet();

  for (const file of cfg.files) {
    const url = `https://huggingface.co/${cfg.repo}/resolve/main/${file.path}`;
    const finalDest = `${destDir}/${file.dest}`;
    try {
      console.log(`  ↓ ${file.path} -> ${finalDest}`);
      await downloadFile(url, finalDest, file.sha, token);
    } catch (e) {
      console.error(
        `  ✗ ${file.path}: ${e instanceof Error ? e.message : String(e)}`,
      );
      process.exit(1);
    }
  }

  console.log(`\nDone. Check: ls -lh ${destDir}/`);
  const ls = await Bun.$`ls -lh ${destDir}`.text();
  console.log(ls);

  // The app treats a variant as installed only when every file is present.
  const missing: string[] = [];
  for (const file of cfg.files) {
    if (!(await Bun.file(`${destDir}/${file.dest}`).exists())) {
      missing.push(file.dest);
    }
  }
  if (missing.length === 0) {
    console.log(`\n✓ Ready for: bun run tauri dev (select "${variant}")`);
  } else {
    console.log(
      `\n⚠ Missing: ${missing.join(", ")} — the app will run in MOCK mode.`,
    );
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
