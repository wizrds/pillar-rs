# Changelog

---
## [0.2.2](https://github.com/wizrds/pillar-rs/-/compare/0.2.1...0.2.2) - 2026-05-09

### Bug Fixes

- **(duckdb)** Wrap Merge aggregates in ANY_VALUE and remove GROUP BY suppression - ([08559af](https://github.com/wizrds/pillar-rs/-/commit/08559af6bc4da31d117acf645e65f01da6607fe5)) - Timothy Pogue
---
## [0.2.1](https://github.com/wizrds/pillar-rs/-/compare/0.2.0...0.2.1) - 2026-05-09

### Bug Fixes

- **(macros)** Use fully qualified path to view name method to avoid conflicts - ([1855ea0](https://github.com/wizrds/pillar-rs/-/commit/1855ea0d43939288532f4a43731d87d3f3395ee8)) - Timothy Pogue

### Miscellaneous

- Update README - ([0f9de97](https://github.com/wizrds/pillar-rs/-/commit/0f9de97a8cdd8e2977f57b5a12f73884a3b3a75f)) - Timothy Pogue
---
## [0.2.0](https://github.com/wizrds/pillar-rs/-/compare/0.1.2...0.2.0) - 2026-05-08

### Features

- Change material views to views and add 'materialized' option to macro - ([a1eee0d](https://github.com/wizrds/pillar-rs/-/commit/a1eee0dd231fa8b8be775c4409f6a423b58d33eb)) - Timothy Pogue

### Miscellaneous

- Update README - ([5520d8b](https://github.com/wizrds/pillar-rs/-/commit/5520d8b83365cb4d3acd0326f996595a2cf57bcb)) - Timothy Pogue
---
## [0.1.2](https://github.com/wizrds/pillar-rs/-/compare/0.1.1...0.1.2) - 2026-05-08

### Bug Fixes

- Ensure variants of smart pointers covered in trait - ([2feb7af](https://github.com/wizrds/pillar-rs/-/commit/2feb7af473e2ea693b9b05c2967e936e269eab23)) - Timothy Pogue

### Miscellaneous

- Update description - ([d53bd37](https://github.com/wizrds/pillar-rs/-/commit/d53bd3748815dbb32950af60402c519b7632c268)) - Timothy Pogue
---
## [0.1.1](https://github.com/wizrds/pillar-rs/-/compare/0.1.0...0.1.1) - 2026-05-08

### Bug Fixes

- Ensure arc wrapped databases fulfill database trait - ([101f957](https://github.com/wizrds/pillar-rs/-/commit/101f9576d961e6cb6e3b35c8070c47bcb4682de1)) - Timothy Pogue
---
## [0.1.0] - 2026-05-08

### Bug Fixes

- Add ColumnRef type instead of relying on trait and strings only - ([d56eac5](https://github.com/wizrds/pillar-rs/-/commit/d56eac5c690fc01b3fc303c29961829f65d0a13d)) - Timothy Pogue
- Add ast fluent API - ([1fd75e1](https://github.com/wizrds/pillar-rs/-/commit/1fd75e133baddbc917fb1a5c144cf7dca5948c4b)) - Timothy Pogue
- Fix insert serialization path and column issues in clickhouse crate - ([fb4f3c9](https://github.com/wizrds/pillar-rs/-/commit/fb4f3c9d264d68cdc0511f02165e68083806e8db)) - Timothy Pogue
- Explicitly map column types to arrow types - ([1663b88](https://github.com/wizrds/pillar-rs/-/commit/1663b8800ad2fae9714b0898dcd726a007e41fdf)) - Timothy Pogue
- Improve queries in migrations table in runner and fix streaming results in clickhouse - ([4dfc74d](https://github.com/wizrds/pillar-rs/-/commit/4dfc74d1e449d9b40d429283ca67f1fadd9f93a4)) - Timothy Pogue
- Add timezone normalization in duckdb impl for serde_arrow interop - ([a96bf2c](https://github.com/wizrds/pillar-rs/-/commit/a96bf2cac94684ec792ac10bb408980e6bc528f5)) - Timothy Pogue
- Fix schema trait impls in macros and add aliased projections - ([d587c69](https://github.com/wizrds/pillar-rs/-/commit/d587c69aad29f3b64797596d0e5651c845f2fd77)) - Timothy Pogue
- Improve macros and add additional attribute options - ([286bcaa](https://github.com/wizrds/pillar-rs/-/commit/286bcaa139f6a2a31282e41812acb4158fb32033)) - Timothy Pogue
- Fix macros and support custom column types - ([dc27ca8](https://github.com/wizrds/pillar-rs/-/commit/dc27ca841e87ab5c33191cc72905141b7a97d595)) - Timothy Pogue
- Reorganize ast in core crate and add README - ([ae4611b](https://github.com/wizrds/pillar-rs/-/commit/ae4611b5feb873a1abca61e2a434f2ab53caaef9)) - Timothy Pogue
- Improve generated code for models to provide more intuitive usage - ([f2f6701](https://github.com/wizrds/pillar-rs/-/commit/f2f6701506b55c82553dfeb40b2301619384dee6)) - Timothy Pogue
- Add better support for materialized views agnostic to the backend - ([df8ecc1](https://github.com/wizrds/pillar-rs/-/commit/df8ecc1cdc43fec41864adaee023afa95fe39065)) - Timothy Pogue
- Small fixes and improvements - ([e24d7e4](https://github.com/wizrds/pillar-rs/-/commit/e24d7e4823efbe149ca28a793c782c41d21123e8)) - Timothy Pogue
- Add derive macros and re-exports through meta crate - ([b380403](https://github.com/wizrds/pillar-rs/-/commit/b380403fbff5b4aeb256d932c44ec49bbb44a716)) - Timothy Pogue
- Finish initial core contract - ([310196e](https://github.com/wizrds/pillar-rs/-/commit/310196e4592e9ce22d9a600e13412a78e00b1ddb)) - Timothy Pogue

### Features

- Add CTEs, window functions, subqueries, UNION, INSERT SELECT, and RETURNING - ([9888c3c](https://github.com/wizrds/pillar-rs/-/commit/9888c3c08fb5d90a54980f1e859724696ee8f512)) - Timothy Pogue
- Add query result type, ensure migrations info is persisted, add table exists statement - ([9dcd3ca](https://github.com/wizrds/pillar-rs/-/commit/9dcd3caf6647fbea48c09a58a8ffcbd666f4bc88)) - Timothy Pogue
- Add structured column types, TTL, and aggregate state/merge support - ([14389a5](https://github.com/wizrds/pillar-rs/-/commit/14389a5a8aee65f79a3ee30d0a1804104e1f590c)) - Timothy Pogue
- Add clickhouse implementation - ([7c4bd7a](https://github.com/wizrds/pillar-rs/-/commit/7c4bd7aa17945fc1cee05a38693ad22cddbcd113)) - Timothy Pogue
- Add duckdb implementation - ([a00a4d8](https://github.com/wizrds/pillar-rs/-/commit/a00a4d89b173cd82c83e8af1d56e4ca7a270ee2b)) - Timothy Pogue
- Initial project setup :tada: - ([6e99f3b](https://github.com/wizrds/pillar-rs/-/commit/6e99f3b9f0dda44ec79e4deca7f65aa9950f9259)) - Timothy Pogue

### Miscellaneous

- Fix typos in various files and remove excess - ([6bd5c1c](https://github.com/wizrds/pillar-rs/-/commit/6bd5c1ced2da186d23018d8da2faeb853272fe82)) - Timothy Pogue
- Update README - ([e36420d](https://github.com/wizrds/pillar-rs/-/commit/e36420de4a410b2a7277b2054c245ce9b3d2386f)) - Timothy Pogue
- Simplify migrations table queries in migration runner - ([e012eab](https://github.com/wizrds/pillar-rs/-/commit/e012eab8738b0b1880edd3f9020aecf33fe742ad)) - Timothy Pogue
- Update README - ([7331e4c](https://github.com/wizrds/pillar-rs/-/commit/7331e4cae3b674eaaac7fa9c222f27a7688697ef)) - Timothy Pogue
- Add docstrings to relevant public APIs - ([6a8b05b](https://github.com/wizrds/pillar-rs/-/commit/6a8b05b36f2bd76226a6c6db9d47e903bb99a2bb)) - Timothy Pogue
- Reorganize macros crate - ([d812365](https://github.com/wizrds/pillar-rs/-/commit/d8123656392893e24c17d445aa2820fad757003b)) - Timothy Pogue
- Reorganize modules in core - ([587f1a9](https://github.com/wizrds/pillar-rs/-/commit/587f1a99ad064a60689ce9fc86ed97739f9f41c9)) - Timothy Pogue
- Refactor traits.rs into separate files - ([291270b](https://github.com/wizrds/pillar-rs/-/commit/291270bd69c9b2208d72bcbbcd2030c1d8065549)) - Timothy Pogue

