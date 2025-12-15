# FunscriptVideo (FSV)

> [!CAUTION]
> This file format is a proposal and is **not supported by any major players or vendors**.  
> It is currently maintained for **personal use only**.  
> To use it with other applications, you must **extract its contents manually or via the provided CLI tool**.  
> There is **no guarantee of third-party tooling or future compatibility** at this time.

## Overview

**FunScriptVideo (FSV)** is a **ZIP-based container format** (`.fsv`) designed to bundle synchronized **videos**, **FunScripts**, and related **metadata** into a single, portable file.
Its purpose is to simplify distribution, maintain proper attribution, and ensure that multi-file interactive media can be shared as a single cohesive package.

The repository includes:

- The **FSV specification** — detailing the required structure, fields, and validation rules. ([here](SPEC.md))
- A **CLI tool** for creating, validating, and extracting `.fsv` files for personal or development use.

## Key Features

- Bundles multiple videos and FunScripts with shared metadata
- Uses a clear, JSON-based manifest (`metadata.json`)
- Ensures portability, integrity, and creator attribution
- Supports optional previews and subtitles
