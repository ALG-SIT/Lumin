//! Single source of truth for the ONNX model variants Lumin can install.
//!
//! Download specs, on-disk layout, and the inference wiring are all derived
//! from [`VARIANTS`], so adding a model means adding one entry here.

use super::tokenizer::ChatFormat;
use std::path::{Path, PathBuf};

/// How a variant's ONNX graphs have to be driven during generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    /// One decoder graph that takes `input_ids` directly (Gemma 3 1B).
    DecoderOnly,
    /// An `embed_tokens` graph feeds `inputs_embeds` (and, for Gemma 3n/4,
    /// `per_layer_inputs`) into the decoder graph.
    EmbedChained,
}

/// One file that makes up a variant.
#[derive(Debug, Clone, Copy)]
pub struct FileSpec {
    /// Path inside the Hugging Face repo.
    pub url_path: &'static str,
    /// Filename on disk, relative to the variant's model directory.
    ///
    /// External-data files MUST keep their upstream name: the `.onnx` graph
    /// references them by the literal filename recorded at export time.
    pub dest_name: &'static str,
    /// Expected SHA256 (lowercase hex), taken from the HF API's `lfs.oid`.
    pub expected_sha256: Option<&'static str>,
    /// Upstream size in bytes, used to show a download estimate before install.
    pub size_bytes: u64,
}

/// A downloadable, selectable model.
#[derive(Debug, Clone, Copy)]
pub struct Variant {
    /// Stable id used by the frontend and persisted as the active selection.
    pub id: &'static str,
    /// Human-readable name shown in the model manager.
    pub display_name: &'static str,
    /// Model family, used to group entries in the UI.
    pub family: &'static str,
    pub quantization: &'static str,
    pub description: &'static str,
    pub repo: &'static str,
    pub architecture: Architecture,
    /// Turn markers this variant was instruction-tuned with. Families are not
    /// interchangeable; see [`ChatFormat`].
    pub chat_format: ChatFormat,
    /// Subdirectory under the model root, or `""` to use the root itself.
    ///
    /// Variants that ship external data under generic upstream names (every
    /// Gemma 3n / Gemma 4 build calls it `decoder_model_merged_q4.onnx_data`)
    /// need their own directory so installing one does not overwrite another.
    pub dir_name: &'static str,
    /// The decoder graph, relative to the variant directory.
    pub decoder_file: &'static str,
    /// The `embed_tokens` graph for [`Architecture::EmbedChained`] variants.
    pub embed_file: Option<&'static str>,
    pub tokenizer_file: &'static str,
    pub files: &'static [FileSpec],
    /// Physical RAM in GB below which the variant is not recommended.
    pub min_memory_gb: u64,
}

impl Variant {
    /// Directory holding this variant's files.
    pub fn dir(&self, model_root: &Path) -> PathBuf {
        if self.dir_name.is_empty() {
            model_root.to_path_buf()
        } else {
            model_root.join(self.dir_name)
        }
    }

    pub fn decoder_path(&self, model_root: &Path) -> PathBuf {
        self.dir(model_root).join(self.decoder_file)
    }

    pub fn embed_path(&self, model_root: &Path) -> Option<PathBuf> {
        self.embed_file.map(|f| self.dir(model_root).join(f))
    }

    pub fn tokenizer_path(&self, model_root: &Path) -> PathBuf {
        self.dir(model_root).join(self.tokenizer_file)
    }

    /// Total download size of every file in the variant.
    pub fn download_size_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).sum()
    }

    /// True once every file of the variant is present on disk.
    pub fn is_installed(&self, model_root: &Path) -> bool {
        let dir = self.dir(model_root);
        self.files.iter().all(|f| dir.join(f.dest_name).exists())
    }

    /// Bytes actually on disk for this variant (partial installs included).
    pub fn installed_size_bytes(&self, model_root: &Path) -> u64 {
        let dir = self.dir(model_root);
        self.files
            .iter()
            .filter_map(|f| std::fs::metadata(dir.join(f.dest_name)).ok())
            .map(|m| m.len())
            .sum()
    }
}

/// Variant selected when nothing has been chosen yet.
pub const DEFAULT_VARIANT_ID: &str = "1b-int4";

/// SHA256 of the Gemma 3 1B tokenizer, verified against the HF API (`lfs.oid`).
pub const SHA_1B_TOKENIZER: &str =
    "55da1312bdf1d7d8fe8d9d1b3eed04086261149e6034e0ac3f8c633b67f5aac8";

/// SHA256 of the Gemma 3 1B INT4 decoder graph.
pub const SHA_1B_INT4_ONNX: &str =
    "69686023e5892376e38fcbcdd0c77af432c55b3bcd03aee6d561bd1f04507da0";

/// Gemma 4 E2B and E4B ship byte-identical tokenizers.
const SHA_GEMMA4_TOKENIZER: &str =
    "47bd35616c7c782aaca6ccf48c75f3461d5877170984b8836b375107d0a9f566";
const SIZE_GEMMA4_TOKENIZER: u64 = 19_439_251;

/// Every installable model, in the order the model manager shows them.
pub static VARIANTS: &[Variant] = &[
    Variant {
        id: "1b-int4",
        display_name: "Gemma 3 1B INT4",
        family: "Gemma 3",
        quantization: "INT4",
        description: "軽量・高速。まず試すならこれ。",
        repo: "onnx-community/gemma-3-1b-it-ONNX",
        architecture: Architecture::DecoderOnly,
        chat_format: ChatFormat::GemmaTurn,
        // Installed before per-variant directories existed; kept flat so
        // existing installs keep working without a migration.
        dir_name: "",
        decoder_file: "gemma-3-1b-it-int4.onnx",
        embed_file: None,
        tokenizer_file: "tokenizer.json",
        min_memory_gb: 8,
        files: &[
            // The repo has no model_int4.*; the INT4 build is published as q4
            // (MatMulNBits 4-bit).
            FileSpec {
                url_path: "onnx/model_q4.onnx",
                dest_name: "gemma-3-1b-it-int4.onnx",
                expected_sha256: Some(SHA_1B_INT4_ONNX),
                size_bytes: 347_363,
            },
            FileSpec {
                url_path: "onnx/model_q4.onnx_data",
                dest_name: "model_q4.onnx_data",
                expected_sha256: Some(
                    "c2370070be257a98d50e17d81be13e18304c39e7e6d9d1416f8f883681d2a17b",
                ),
                size_bytes: 859_106_816,
            },
            FileSpec {
                url_path: "tokenizer.json",
                dest_name: "tokenizer.json",
                expected_sha256: Some(SHA_1B_TOKENIZER),
                size_bytes: 20_323_013,
            },
        ],
    },
    Variant {
        id: "1b-int8",
        display_name: "Gemma 3 1B INT8",
        family: "Gemma 3",
        quantization: "INT8",
        description: "INT4 が不安定な環境向けのフォールバック。",
        repo: "onnx-community/gemma-3-1b-it-ONNX",
        architecture: Architecture::DecoderOnly,
        chat_format: ChatFormat::GemmaTurn,
        dir_name: "",
        decoder_file: "gemma-3-1b-it-int8.onnx",
        embed_file: None,
        tokenizer_file: "tokenizer.json",
        min_memory_gb: 8,
        files: &[
            // int8 is a single-file graph; the repo has no model_int8.onnx_data.
            FileSpec {
                url_path: "onnx/model_int8.onnx",
                dest_name: "gemma-3-1b-it-int8.onnx",
                expected_sha256: Some(
                    "6d8ddeb9c637d43625df45933ad3a9e2337b8a027ab37a70dc230735ba285f5c",
                ),
                size_bytes: 1_001_481_982,
            },
            FileSpec {
                url_path: "tokenizer.json",
                dest_name: "tokenizer.json",
                expected_sha256: Some(SHA_1B_TOKENIZER),
                size_bytes: 20_323_013,
            },
        ],
    },
    Variant {
        id: "3n-e2b-int4",
        display_name: "Gemma 3n E2B INT4",
        family: "Gemma 3n",
        quantization: "INT4",
        description: "モバイル最適化 (PLE + MatFormer)。",
        repo: "onnx-community/gemma-3n-E2B-it-ONNX",
        architecture: Architecture::EmbedChained,
        chat_format: ChatFormat::GemmaTurn,
        // Its tokenizer differs from Gemma 3's while sharing the filename, so
        // a flat layout would have the two variants clobber each other.
        dir_name: "gemma-3n-e2b-int4",
        decoder_file: "decoder_model_merged_q4.onnx",
        embed_file: Some("embed_tokens_q4.onnx"),
        tokenizer_file: "tokenizer.json",
        min_memory_gb: 16,
        files: &[
            FileSpec {
                url_path: "onnx/decoder_model_merged_q4.onnx",
                dest_name: "decoder_model_merged_q4.onnx",
                expected_sha256: Some(
                    "4fcb3a37937db577756270c504851e9366ffa738ace6c5ee7d345728aa8dcbd0",
                ),
                size_bytes: 1_686_685,
            },
            FileSpec {
                url_path: "onnx/decoder_model_merged_q4.onnx_data",
                dest_name: "decoder_model_merged_q4.onnx_data",
                expected_sha256: Some(
                    "297a9301058969f1e67e42546a48875b4250f58b10a28249ff08d76e0b5ead57",
                ),
                size_bytes: 1_620_499_456,
            },
            // The decoder takes inputs_embeds, so the embed graph is required
            // rather than optional.
            FileSpec {
                url_path: "onnx/embed_tokens_q4.onnx",
                dest_name: "embed_tokens_q4.onnx",
                expected_sha256: Some(
                    "54431e64a782ed74220a0af6b4184a6bf59e4b47e0b14ea4e1335810bc0f4f9b",
                ),
                size_bytes: 3_413,
            },
            FileSpec {
                url_path: "onnx/embed_tokens_q4.onnx_data",
                dest_name: "embed_tokens_q4.onnx_data",
                expected_sha256: Some(
                    "a94d38d81555f7143d1f2108433f866aa54cd4b574e3a527e3edacc34c475d70",
                ),
                size_bytes: 1_634_017_280,
            },
            FileSpec {
                url_path: "tokenizer.json",
                dest_name: "tokenizer.json",
                expected_sha256: Some(
                    "44cb3d7d545cf895311e004d9a2b2ce823be5eb84c9aa31f73858b607c44c924",
                ),
                size_bytes: 20_366_294,
            },
        ],
    },
    Variant {
        id: "4-e2b-int4",
        display_name: "Gemma 4 E2B INT4",
        family: "Gemma 4",
        quantization: "INT4",
        description: "Gemma 4 の軽量版。教室端末での常用を想定。",
        repo: "onnx-community/gemma-4-E2B-it-ONNX",
        architecture: Architecture::EmbedChained,
        chat_format: ChatFormat::Gemma4Turn,
        dir_name: "gemma-4-e2b-int4",
        decoder_file: "decoder_model_merged_q4.onnx",
        embed_file: Some("embed_tokens_q4.onnx"),
        tokenizer_file: "tokenizer.json",
        min_memory_gb: 16,
        files: &[
            FileSpec {
                url_path: "onnx/decoder_model_merged_q4.onnx",
                dest_name: "decoder_model_merged_q4.onnx",
                expected_sha256: Some(
                    "c6edb929bf342c524728d37efd400285ee71525e8fe64ff996341f78c3e577d2",
                ),
                size_bytes: 647_599,
            },
            FileSpec {
                url_path: "onnx/decoder_model_merged_q4.onnx_data",
                dest_name: "decoder_model_merged_q4.onnx_data",
                expected_sha256: Some(
                    "b879fe4b946c9b9ff6acb60f7c5eda3d2c9c4df8625895feb2d1e269002f0345",
                ),
                size_bytes: 1_864_102_912,
            },
            FileSpec {
                url_path: "onnx/embed_tokens_q4.onnx",
                dest_name: "embed_tokens_q4.onnx",
                expected_sha256: Some(
                    "2d8c8a2bcc30e8ded7f636967c2a58a346116583356dd933720b005fc88079c4",
                ),
                size_bytes: 5_142,
            },
            FileSpec {
                url_path: "onnx/embed_tokens_q4.onnx_data",
                dest_name: "embed_tokens_q4.onnx_data",
                expected_sha256: Some(
                    "40fa957d9988b8a0160c8b0eb5c3f781a237627e9f7153f30514a4ffb2e62888",
                ),
                size_bytes: 1_762_656_256,
            },
            FileSpec {
                url_path: "tokenizer.json",
                dest_name: "tokenizer.json",
                expected_sha256: Some(SHA_GEMMA4_TOKENIZER),
                size_bytes: SIZE_GEMMA4_TOKENIZER,
            },
        ],
    },
    Variant {
        id: "4-e4b-int4",
        display_name: "Gemma 4 E4B INT4",
        family: "Gemma 4",
        quantization: "INT4",
        description: "Gemma 4 の高精度版。メモリに余裕のある教師端末向け。",
        repo: "onnx-community/gemma-4-E4B-it-ONNX",
        architecture: Architecture::EmbedChained,
        chat_format: ChatFormat::Gemma4Turn,
        dir_name: "gemma-4-e4b-int4",
        decoder_file: "decoder_model_merged_q4.onnx",
        embed_file: Some("embed_tokens_q4.onnx"),
        tokenizer_file: "tokenizer.json",
        min_memory_gb: 24,
        files: &[
            FileSpec {
                url_path: "onnx/decoder_model_merged_q4.onnx",
                dest_name: "decoder_model_merged_q4.onnx",
                expected_sha256: Some(
                    "af764e3b468255401694cf9acbdf859e7e2febd8cd9abd1443ab8ab5493d1f99",
                ),
                size_bytes: 814_829,
            },
            FileSpec {
                url_path: "onnx/decoder_model_merged_q4.onnx_data",
                dest_name: "decoder_model_merged_q4.onnx_data",
                expected_sha256: Some(
                    "3f0dda0cd6b575a49c9a8753250256518a2b9a053ed4cc9dddb6e228a3732099",
                ),
                size_bytes: 2_093_703_168,
            },
            // E4B's decoder weights exceed the 2 GiB protobuf limit and are
            // split across two external-data shards.
            FileSpec {
                url_path: "onnx/decoder_model_merged_q4.onnx_data_1",
                dest_name: "decoder_model_merged_q4.onnx_data_1",
                expected_sha256: Some(
                    "3ce62ea330a5549d791d9b7a2a72dcad8cd4f4083fac490b72b1d539f46f07eb",
                ),
                size_bytes: 1_286_205_440,
            },
            FileSpec {
                url_path: "onnx/embed_tokens_q4.onnx",
                dest_name: "embed_tokens_q4.onnx",
                expected_sha256: Some(
                    "78f0c7025bc90339807336da699d7dac6c51ddbb4a690aa4c9e68f3e42b2b353",
                ),
                size_bytes: 5_134,
            },
            FileSpec {
                url_path: "onnx/embed_tokens_q4.onnx_data",
                dest_name: "embed_tokens_q4.onnx_data",
                expected_sha256: Some(
                    "67a3100d70c26840c34f8aac29aeec3db31315268cfc53dbc456e2a648849f35",
                ),
                size_bytes: 1_839_202_304,
            },
            FileSpec {
                url_path: "onnx/embed_tokens_q4.onnx_data_1",
                dest_name: "embed_tokens_q4.onnx_data_1",
                expected_sha256: Some(
                    "260b86f8e306ea73aeb8f85c0c142e1fef4a5ac1889a2345e35891518d9367d5",
                ),
                size_bytes: 396_361_728,
            },
            FileSpec {
                url_path: "tokenizer.json",
                dest_name: "tokenizer.json",
                expected_sha256: Some(SHA_GEMMA4_TOKENIZER),
                size_bytes: SIZE_GEMMA4_TOKENIZER,
            },
        ],
    },
];

/// Look up a variant by id. `"default"` resolves to [`DEFAULT_VARIANT_ID`].
pub fn find(id: &str) -> Option<&'static Variant> {
    let id = if id == "default" {
        DEFAULT_VARIANT_ID
    } else {
        id
    };
    VARIANTS.iter().find(|v| v.id == id)
}

/// Comma-separated list of valid ids, for error messages.
pub fn known_ids() -> String {
    VARIANTS.iter().map(|v| v.id).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = VARIANTS.iter().map(|v| v.id).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "duplicate variant id in the catalog");
    }

    #[test]
    fn gemma4_variants_are_present() {
        let e2b = find("4-e2b-int4").expect("Gemma 4 E2B must be installable");
        let e4b = find("4-e4b-int4").expect("Gemma 4 E4B must be installable");

        for v in [e2b, e4b] {
            assert_eq!(v.family, "Gemma 4");
            assert_eq!(v.architecture, Architecture::EmbedChained);
            assert!(
                v.embed_file.is_some(),
                "{} must ship an embed_tokens graph",
                v.id
            );
        }
        assert!(
            e4b.min_memory_gb > e2b.min_memory_gb,
            "E4B is the larger model and must ask for more RAM"
        );
        assert!(e4b.download_size_bytes() > e2b.download_size_bytes());
    }

    #[test]
    fn default_variant_resolves() {
        assert_eq!(find("default").unwrap().id, DEFAULT_VARIANT_ID);
        assert_eq!(find(DEFAULT_VARIANT_ID).unwrap().id, DEFAULT_VARIANT_ID);
        assert!(find("does-not-exist").is_none());
    }

    #[test]
    fn every_variant_declares_the_files_it_loads() {
        for v in VARIANTS {
            let names: Vec<&str> = v.files.iter().map(|f| f.dest_name).collect();
            assert!(
                names.contains(&v.decoder_file),
                "{}: decoder {} is not in the download list",
                v.id,
                v.decoder_file
            );
            assert!(
                names.contains(&v.tokenizer_file),
                "{}: tokenizer is not in the download list",
                v.id
            );
            if let Some(embed) = v.embed_file {
                assert!(
                    names.contains(&embed),
                    "{}: embed graph {embed} is not in the download list",
                    v.id
                );
            }
        }
    }

    /// External data is referenced by literal filename inside the `.onnx`
    /// graph, so two variants sharing a directory must not share a filename.
    #[test]
    fn variants_sharing_a_directory_do_not_share_filenames() {
        for (i, a) in VARIANTS.iter().enumerate() {
            for b in VARIANTS.iter().skip(i + 1) {
                if a.dir_name != b.dir_name {
                    continue;
                }
                for fa in a.files {
                    for fb in b.files {
                        if fa.dest_name != fb.dest_name {
                            continue;
                        }
                        assert_eq!(
                            fa.expected_sha256, fb.expected_sha256,
                            "{} and {} both write {} into the same directory with different content",
                            a.id, b.id, fa.dest_name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn paths_are_scoped_to_the_variant_directory() {
        let root = Path::new("/models");
        let e2b = find("4-e2b-int4").unwrap();
        assert_eq!(
            e2b.decoder_path(root),
            Path::new("/models/gemma-4-e2b-int4/decoder_model_merged_q4.onnx")
        );
        assert_eq!(
            e2b.embed_path(root).unwrap(),
            Path::new("/models/gemma-4-e2b-int4/embed_tokens_q4.onnx")
        );

        // The legacy variant stays flat so existing installs keep working.
        let legacy = find("1b-int4").unwrap();
        assert_eq!(
            legacy.decoder_path(root),
            Path::new("/models/gemma-3-1b-it-int4.onnx")
        );
    }
}
