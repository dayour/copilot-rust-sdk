---
description: Read-only AC3 validation and optional darbot analysis on native Windows MXC.
on:
  workflow_dispatch:
    inputs:
      analyze:
        description: Run the no-tool darbot analysis after contained validation
        type: boolean
        default: false
permissions:
  contents: read
concurrency:
  job-discriminator: ${{ github.run_id }}
engine: copilot
imports:
  - .github/agents/darbot.agent.md
runs-on: [darbot-windows-mxc]
tools: {}
metadata:
  native-backend: windows-mxc
  native-profile: ac3-readonly-v1
---

# darbot

Validate this repository's portable AC3 card in Windows MXC before any analysis.
When analysis is requested, report only the supplied, validated snapshot.
Distinguish failures, unavailable observations, intentional privacy exclusions,
and configuration from actual execution. Do not perform repository mutations.

This source is checked by gh-aw. The companion darbot.native.yml is emitted for
the Windows MXC read-only profile, not by the upstream Linux/Docker runtime.
Unsupported general gh-aw capabilities are not implied by this pilot.
