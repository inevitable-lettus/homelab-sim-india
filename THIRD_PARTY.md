# Third-party code

Every file copied or adapted from another project is listed here, with its origin path and commit. Copied files keep their original notices.

| File here | Origin project | Origin path | Commit | Licence | Changes |
|---|---|---|---|---|---|
| _None yet._ | | | | | |

npm and Cargo dependencies are tracked by their lockfiles and are not listed here.

## The terminal's Linux image

The optional terminal downloads a Linux system that is built by `tools/guest/` and is not stored in this repository. It is distributed with the site, so its licences apply to the site:

- **Alpine Linux** (`i386/alpine`, version pinned in `tools/guest/versions.env`) with `wireguard-tools`, `openssh-client-default`, `iproute2`, `curl` and `nano`, unmodified. Many packages are GPL or LGPL.
- **v86** (BSD-2-Clause), the emulator, from the `v86` npm package at the commit pinned in `tools/guest/versions.env`.

Every build writes `guest/SOURCES.md` next to the image: the full package list with versions and licences, where each package's source is, and a written offer for the corresponding source. The site's credits page links to it. The offer is made in the project's name, so whoever publishes a copy of the site takes it on.
