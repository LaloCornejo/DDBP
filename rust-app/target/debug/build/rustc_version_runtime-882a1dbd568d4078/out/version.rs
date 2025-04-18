
            /// Returns the `rustc` SemVer version and additional metadata
            /// like the git short hash and build date.
            pub fn version_meta() -> VersionMeta {
                VersionMeta {
                    semver: Version {
                        major: 1,
                        minor: 85,
                        patch: 1,
                        pre: Prerelease::new("").unwrap(),
                        build: BuildMetadata::new("").unwrap(),
                    },
                    host: "x86_64-apple-darwin".to_owned(),
                    short_version_string: "rustc 1.85.1 (4eb161250 2025-03-15)".to_owned(),
                    commit_hash: Some("4eb161250e340c8f48f66e2b929ef4a5bed7c181".to_owned()),
                    commit_date: Some("2025-03-15".to_owned()),
                    build_date: None,
                    channel: Channel::Stable,
                    llvm_version: Some(LlvmVersion{ major: 19, minor: 1 }),
                }
            }
            