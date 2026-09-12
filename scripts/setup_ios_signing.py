#!/usr/bin/env python3
"""Write the untracked iOS signing files for the developer running the build.

Code signing is per-developer, so no signing identity belongs in a tracked
file. This puts the team ID from `$APPLE_DEVELOPMENT_TEAM` into the two places
an iOS build actually reads it:

- `src-tauri/tauri.ios.conf.json`, which Tauri merges into tauri.conf.json for
  iOS builds. The environment variable alone is not enough: Xcode's "Build Rust
  Code" phase shells out to `tauri ios xcode-script`, which re-reads the config
  in a sanitized environment and otherwise fails with "`apple.development-team`
  is empty".
- `gen/apple/ExportOptions.plist`, if the Xcode project has been generated.
  `tauri ios init` writes one that only sets `method`, and
  `xcodebuild -exportArchive` then refuses the app with "requires a
  provisioning profile". Tauri has no configuration for this file, so it is
  patched here rather than by hand. The profile name is the one Xcode's
  automatic signing creates for any team.

Both files are gitignored. Run this before `tauri ios init` (so the generated
project gets the team) and again after it (to patch the export options); the
`ios:init` script does both.
"""
import argparse
import json
import os
from pathlib import Path
import plistlib

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "src-tauri/tauri.conf.json"
LOCAL_CONFIG = ROOT / "src-tauri/tauri.ios.conf.json"
EXPORT_OPTIONS = ROOT / "src-tauri/gen/apple/ExportOptions.plist"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--team",
        default=os.environ.get("APPLE_DEVELOPMENT_TEAM", ""),
        help="Apple Developer team ID (default: $APPLE_DEVELOPMENT_TEAM)",
    )
    args = parser.parse_args()
    if not args.team:
        parser.error(
            "No signing team. Set APPLE_DEVELOPMENT_TEAM to your Apple Developer "
            "team ID, which is the OU field of your signing certificate:\n"
            "  security find-identity -v -p codesigning\n"
            "  security find-certificate -c 'Apple Development' -p | openssl x509 -noout -subject"
        )

    identifier = json.loads(CONFIG.read_text())["identifier"]

    LOCAL_CONFIG.write_text(
        json.dumps(
            {
                "$schema": "https://schema.tauri.app/config/2",
                "bundle": {"iOS": {"developmentTeam": args.team}},
            },
            indent=2,
        )
        + "\n"
    )
    print(f"{LOCAL_CONFIG.relative_to(ROOT)}: development team {args.team}")

    if not EXPORT_OPTIONS.is_file():
        print(f"{EXPORT_OPTIONS.relative_to(ROOT)}: not generated yet, skipping")
        return
    options = plistlib.loads(EXPORT_OPTIONS.read_bytes())
    options.update(
        {
            "teamID": args.team,
            "signingStyle": "automatic",
            "provisioningProfiles": {identifier: f"iOS Team Provisioning Profile: {identifier}"},
        }
    )
    EXPORT_OPTIONS.write_bytes(plistlib.dumps(options))
    print(f"{EXPORT_OPTIONS.relative_to(ROOT)}: automatic signing for {identifier}")


if __name__ == "__main__":
    main()
