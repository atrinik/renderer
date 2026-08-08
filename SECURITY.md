# Security policy

Report vulnerabilities privately through GitHub's security advisory interface.
Do not include private content, credentials, or player data in a report.

Scenes and resources are untrusted. Identity, revision, digest, dimensions,
counts, scalar values, and total pixels are validated before allocation or GPU
submission. Resource access is capability-based and bounded; renderer libraries
perform no ambient filesystem or network access. Output never overwrites an
existing path. Unsafe code is forbidden except for the isolated, documented SDL
raw-handle adapter required by wgpu.
