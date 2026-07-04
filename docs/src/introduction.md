# Introduction

`file-handle` is a lean Rust library for desktop file actions: opening paths
with default applications, revealing paths in file managers, opening terminals,
and moving paths to the system trash.

The crate is feature-gated and dependency-conscious. The default build enables
no operations; callers opt into only the OS actions they need.
