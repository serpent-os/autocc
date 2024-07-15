# autocc

A super simple helper to provide the `/usr/bin/cc` and `/lib/cpp` (via usr-merge) binaries.

Typically this is handled by a symlink, but mutating a fresh transaction for a single symlink seems a bit silly, when we can trivially handle them based on filesystem availability and environmental variables..

For now we're only handling the `cc` case, though may expand in future to support other buildsystem-related problems.

## License

`auto-cc` is available under the terms of the [MPL-2.0](https://spdx.org/licenses/MPL-2.0.html)
