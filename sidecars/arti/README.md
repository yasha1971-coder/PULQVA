# PULQVA Arti sidecar

Pinned upstream binary package: `arti 2.6.0`.

This directory records the Tor sidecar version independently from PULQVA core crates.

T008 proves only that the exact upstream package builds and exposes its proxy CLI on Windows
and Linux. It does not launch Tor or make a Tor network connection.

Upgrade rule: change the version only in a dedicated atomic task with Windows/Linux CI proof.
