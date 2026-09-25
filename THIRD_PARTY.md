# Third-party licensing and release requirements

Apache-2.0 in this repository applies to PULQVA's original work. It does not relicense
third-party software, models, media or downloaded documents.

This is an initial inventory, not a completed distribution-compliance audit.

| Component | Licensing point to verify for each shipped artifact |
| --- | --- |
| FFmpeg | Current pinned asset names include GPL builds. Determine the exact license/version and included libraries from the build; retain licenses, corresponding source and build information as required. Do not describe these binaries as LGPL-only. |
| yt-dlp | Source uses Unlicense, but official PyInstaller executables include GPLv3+ components. Inspect notices and source requirements for the pinned Windows/Linux executables, not just the repository license. |
| Arti, Tauri, Rust dependencies | Collect exact dependency versions, license texts and required notices for the chosen build/features. |
| Future JS runtimes or AI models | Verify redistribution terms before bundling; neither model availability nor a free service implies redistribution rights. |

Before a binary release, record the exact artifact hashes, dependency inventory,
applicable licenses, required notices and corresponding-source delivery mechanism.
The root LICENSE alone does not satisfy these obligations. No binary release is
declared cleared by this document.

Primary sources:
- https://ffmpeg.org/legal.html
- https://github.com/yt-dlp/yt-dlp#licensing
- https://www.apache.org/licenses/LICENSE-2.0

Separate-process integration is not a blanket exemption from distribution obligations.
