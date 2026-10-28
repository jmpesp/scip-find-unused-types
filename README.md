# Look for unused types in a Rust SCIP file.

Generate the index with `rust-analyzer`:

    rust-analyzer scip ${PWD} --output index.scip

Then run this tool:

    scip-find-unused-types \
        --workspace "${PWD}" \
        --index index.scip \
        --filter 'sled'

It will print out a line for each type that it thinks is not used. For example:

    rust-analyzer cargo sled-agent-types 0.1.0 boot_disk/BootDiskUpdatePathParams# | sled-agent/types/src/boot_disk.rs | [17, 11, 35]

This means the tool thinks `BootDiskUpdatePathParams`, defined at
`sled-agent/types/src/boot_disk.rs` line 18, is not used anywhere in the
workspace.

Filtering by crate name is required due to the amount of false positives from
certain crates.

