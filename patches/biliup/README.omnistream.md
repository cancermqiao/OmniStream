# OmniStream Biliup compatibility patch

Source: https://github.com/biliup/biliup
Version: 1.3.5
Commit: 98370ec7cdfbc12003f945f3d529b3e173b1f9e2

The `src/` tree is copied from `crates/biliup` at the pinned commit.
The standalone manifest expands the upstream workspace dependencies and updates
quick-xml from 0.37 to 0.42, which fixes RUSTSEC-2026-0194 and RUSTSEC-2026-0195.
The upstream license is retained in LICENSE.
The RSS parser uses the new text decoder and handles `GeneralRef` events to
preserve escaped and numeric character references with quick-xml 0.42, including
spaces around references.
The password login code uses RSA 0.10 with its required PKCS#1 v1.5 feature and
a Rand 0.10 cryptographic RNG, fixing RUSTSEC-2023-0071. The upstream Rand 0.8
dependency remains for the downloader's existing random-value APIs.

Remove this patch once upstream accepts compatible quick-xml and RSA updates.
