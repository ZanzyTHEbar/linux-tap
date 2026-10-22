# Contributing

Start with AGENTS.md and the ordered backlog. Pick one observable capability and
include evidence appropriate to it. Software simulation, native compilation,
live-node behavior and physical hardware results are separate claims.

New project code is Apache-2.0. Retain upstream notices and record copied source
revisions. Sign off contributions with the Developer Certificate of Origin when
publishing a collaborative repository. No copyright assignment is requested.

Run bash scripts/check.sh before a change is reviewed. Reader changes also need
the PC/SC feature build; bridge changes need scripts/test_nfcd_bridge.py with
the adapter's requirements installed. Physical results must state the exact
model, driver, firmware and trace. Keep credentials out of tests and output.

Do not turn an unresolved SDK limitation into a hidden cloud dependency. Propose
small upstream fixes with a reproducer. Do not announce compatibility, brand
partnerships or a completed security review without evidence.
