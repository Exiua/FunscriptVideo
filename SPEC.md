# FunScriptVideo (FSV) Format Specification

Defines the specification for the **FunScriptVideo** container file format (`.fsv`).

This format allows creators to **bundle one or more synchronized videos and FunScripts** (and optional metadata) into a single, portable file.  
Its goal is to make distributing multi-file content simple, self-contained, and attribution-friendly.

---

## 1. Specification Overview

An `.fsv` file is a **ZIP-based container** that stores:
- One or more **video files**
- One or more **FunScript files**
- A single **metadata file** (`metadata.json`) describing the contents, creators, and relationships between the files

Every `.fsv` archive **MUST** include one `metadata.json` file at its root.  
All other contents are **referenced** by that metadata file.

---

## 2. Terms and Definitions

The key words **"MUST"**, **"MUST NOT"**, **"REQUIRED"**, **"SHALL"**, **"SHALL NOT"**, **"SHOULD"**, **"SHOULD NOT"**, **"RECOMMENDED"**, **"NOT RECOMMENDED"**, **"MAY"**, and **"OPTIONAL"** in this document are to be interpreted as described in  
[BCP 14 / RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) and  
[RFC 8174](https://datatracker.ietf.org/doc/html/rfc8174) when written in all capitals.

**FSV File**  
: A `.fsv` archive that conforms to this specification.

**Video Format**  
: A playable video file referenced by the FSV metadata.

**Script Variant**  
: A FunScript file or logical component that defines motion or interaction data synchronized with the video.

**Creator Metadata**  
: Information describing the original authors, sources, or contributors for each video or script.

**Filestem**  
: The part of a filename before the extension. For example, the filestem of `Normal.mp4` and `Normal.funscript` is `Normal`.

---

## 3. File Layout

An `.fsv` file is a ZIP archive containing (video and script filenames are arbitrary):

```
/
├── metadata.json
├── Normal.mp4
├── Normal.funscript
└── (other optional files)
```

### 3.1 Required Files

| File | Description | Required |
|------|--------------|-----------|
| `metadata.json` | Describes all other contents and their relationships | ✅ |
| Video files | Actual media content referenced in `video_formats` | ✅ (at least one) |
| Script files | FunScript(s) referenced in `script_variants` | ✅ (at least one) |

### 3.2 Optional Files

| File | Description |
|------|--------------|
| Additional axis scripts (`*.roll.funscript`, etc.) | Extra motion data |
| Thumbnails / previews (`*.jpg`, `*.png`) | Optional previews |

---

## 4. Metadata Structure

The `metadata.json` file contains all descriptive and relational information for the FSV archive.

### Example

```json
{
  "format_version": "1.0.0",
  "tags": ["example", "demo", "funscript", "video"],
  "title": "Example FSV Content",
  "creators": {
    "videos": [
      {
        "work_name": "Normal",
        "source_url": "https://example.com/normal_video",
        "creator_info": {
          "name": "John Doe",
          "socials": ["twitter.com/johndoe", "patreon.com/johndoe"]
        }
      }
    ],
    "scripts": [
      {
        "work_name": "Normal",
        "source_url": "https://example.com/normal_script",
        "creator_info": {
          "name": "Alice",
          "socials": ["patreon.com/alice"]
        }
      }
    ]
  },
  "video_formats": [
    {
      "name": "Normal.mp4",
      "description": "Standard 2D video",
      "duration": 123456,
      "start_offset": 0,
      "checksum": "sha256:abcdef..."
    }
  ],
  "script_variants": [
    {
      "name": "Normal",
      "description": "Standard script with optional roll axis",
      "additional_axes": ["roll"],
      "duration": 123456,
      "start_offset": 0,
      "checksum": "sha256:123456..."
    }
  ]
}
```

### 4.1 Root-Level Fields

| Field             | Type             | Description                                      | Required | Default Value               | Error Condition                                 |
| ----------------- | ---------------- | ------------------------------------------------ | -------- | --------------------------- | ----------------------------------------------- |
| `format_version`  | string           | Version of the FSV metadata schema.              | ✅        | *None (must be provided)*   | Missing or not a string → **Invalid container** |
| `tags`            | array of strings | Keywords or categories for discovery.            | ❌        | Empty array (`[]`)          | *None*                                          |
| `title`           | string           | Display name of the content set.                 | ❌        | Filestem of the `.fsv` file | *None*                                          |
| `creators`        | object           | Information about content creators.              | ❌        | Empty object (`{}`)         | *None*                                          |
| `video_formats`   | array            | List of video files referenced in the container. | ✅        | *None (must be provided)*   | Missing or empty array → **Invalid container**  |
| `script_variants` | array            | List of scripts referenced in the container.     | ✅        | *None (must be provided)*   | Missing or empty array → **Invalid container**  |

If a field marked **Required** is missing or of the wrong type, the container is considered to be in an **error state** and **MUST NOT** be treated as valid.  
Fields marked as **Optional** MAY be omitted. When omitted, the value **SHOULD** be assumed to match the **Default Value** column above.

### 4.2 Creators

The `creators` object groups author information for videos and scripts.

| Field     | Type  | Description                                                        |
| --------- | ----- | ------------------------------------------------------------------ |
| `videos`  | array | Each item describes a creator and the work they contributed to.    |
| `scripts` | array | Each item describes a creator and the script variant they created. |

Each creator entry includes:

| Field          | Type   | Description                                            |
| -------------- | ------ | ------------------------------------------------------ |
| `work_name`    | string | Logical name of the work (not necessarily a filename). |
| `source_url`   | string | URL where the content originated.                      |
| `creator_info` | object | Details about the author.                              |

`creator_info` contains:

| Field     | Type   | Description                                      |
| --------- | ------ | ------------------------------------------------ |
| `name`    | string | Display name or alias of the creator.            |
| `socials` | array  | Optional links to social media or support pages. |

### 4.3 Video Formats

| Field         | Type   | Description                                      |
| ------------- | ------ | ------------------------------------------------ |
| `name`        | string | Filename of the video within the container.      |
| `description` | string | Human-readable description (e.g., “3D version”). |
| `checksum`    | string | Hash for integrity verification.                 |

### 4.4 Script Variants

| Field         | Type   | Description                                      |
| ------------- | ------ | ------------------------------------------------ |
| `name`        | string | Filename of the video within the container.      |
| `description` | string | Human-readable description (e.g., “3D version”). |
| `checksum`    | string | Hash for integrity verification.                 |

### 4.5 Subtitle Tracks

The `subtitle_tracks` array defines subtitle or caption files associated with the videos in the container.

Each subtitle entry corresponds to a single subtitle or caption file (e.g., `.srt`, `.smi`, `.vtt`).

| Field             | Type    | Description                                         | Required   |
|------------------ |-------- |---------------------------------------------------- |----------- |
| `name`            | string  | Filename of the subtitle file inside the container. | ✅        |
| `language`        | string  | ISO 639-1 language code (e.g., `"en"`, `"ja"`).     | ✅        |
| `description`     | string  | Human-readable label (e.g., `"English subtitles"`). | ❌        |
| `checksum`        | string  | Hash for integrity verification.                    | ❌        |

Readers **SHOULD** select subtitle tracks by language preference, and **MAY** ignore unsupported formats.

---

## 5. General Rules

- All videos listed in `video_formats` MUST depict the same scene and have matching duration.
- All scripts in `script_variants` MUST synchronize with these videos.
- The `work_name` and `name` fields SHOULD be consistent to allow linking between creator and content entries.
- Any missing optional fields MAY be omitted but must not be set to null.

### 5.1 Incomplete or Deferred-Complete Containers

An FSV file **MAY** omit one or more video files if distribution rights do not permit including those files directly in the archive.

In such cases:

- The `metadata.json` **MUST** still include complete entries for all referenced videos under both `video_formats` and `creators.videos`.
- Each omitted video **SHOULD** include a `description`, `source_url`, and `checksum` (if available) so that players or tools can identify and verify the file once it is obtained separately.
- The container is considered **metadata-valid** but **content-incomplete**.
- Readers and tools **SHOULD** clearly indicate when referenced videos are missing.
- The container **MUST NOT** be treated as fully valid or playable until all files referenced in `video_formats` are present and verified.
- Once all referenced files are included and their checksums match the metadata, the container transitions to a **content-complete** state.

This mechanism allows creators and script authors to distribute FSV files that include full metadata and attributions without violating video distribution restrictions. Tools and consumers can later complete the container by adding the missing video files that match the provided metadata and checksums.

### 5.2 Checksum Format

All checksums used in this specification **MUST** conform to the following format:

```
<algorithm_name>:<hex_encoded_digest>
```

#### 5.2.1 Algorithm Requirements
- The `algorithm_name` **MUST** be written in lowercase ASCII letters (`a–z`), digits (`0–9`), and hyphens (`-`).
- The algorithm **MUST** represent a cryptographically secure, unbroken hash function at the time of publication.
- Implementations **MUST NOT** use obsolete or compromised algorithms (e.g., `md5`, `sha1`).
- The following algorithms are **RECOMMENDED**:
  - `sha256`
  - `sha512`
  - `sha3-256`
- Additional algorithms **MAY** be supported, provided they follow the same naming and format rules.

#### 5.2.2 Digest Encoding
- The digest **MUST** be encoded as a lowercase hexadecimal string with no whitespace.
- The digest **MUST** represent the full output of the hash algorithm (e.g., 64 hex characters for SHA-256).
- Example valid values:
  - `sha256:3d7f7a9d2e3b0caa6abaf93a6f90b8b32a33cd12b2b4e35a3e7762c9a41a73e4`
  - `sha512:bb15f0f7d96c1d6e24ccaa47b62741ad76e79a45d5de2d72a92af7b4df1a6bb2c0ff90e2b066f8f08495d8b93b9c0a42cdb42b5ab33bc3c9d44a51a404210d91`

#### 5.2.3 Validation
A checksum string is considered **valid** if and only if:
1. It matches the regular expression:

```
^[a-z0-9-]+:[a-f0-9]+$
```

2. The algorithm name is supported and not deprecated.
3. The digest length matches the expected bit length of that algorithm.

If any of the above conditions are not met, the container is considered to be in an **invalid state**.

---

## 6. Validation Rules

A container is **invalid** if any of the following conditions are true:

1. `format_version` is missing or empty.
2. `video_formats` is missing or empty.
3. `script_variants` is missing or empty.
4. Any field required by this specification has an unexpected type.

---

## 7. Versioning and Compatibility

- The format_version follows [Semantic Versioning](https://semver.org/) (MAJOR.MINOR.PATCH).
- Readers MAY ignore fields they do not recognize.
- Writers SHOULD NOT remove required fields from previous versions without incrementing the major version

---

## 8. Example Minimal Metadata

```json
{
  "format_version": "1.0.0",
  "video_formats": [{ "name": "video.mp4" }],
  "script_variants": [{ "name": "Main" }]
}
```