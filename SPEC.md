# FunscriptVideo (FSV) Format Specification

The **FunscriptVideo (FSV)** format defines a ZIP-based archive (`.fsv`) for bundling one or more videos, Funscript files, optional subtitle tracks, and associated metadata into a single portable container.

Its goal is to make distributing multi-file content simple, self-contained, and attribution-friendly.

---

## 1. Specification Overview

An `.fsv` file is a **ZIP-based container** that stores:

- One or more **Funscript files**
- Zero or more **video files** present in the archive, but **the metadata MUST declare at least one video entry**
- Zero or more **subtitle files**
- A single **metadata file** (`metadata.json`) describing the contents, creators, and relationships between the files

Every `.fsv` archive **MUST** include exactly one `metadata.json` file at its root.  
All other contents are discovered and validated according to the information in this metadata file.

---

## 2. Terms and Definitions

The key words **"MUST"**, **"MUST NOT"**, **"REQUIRED"**, **"SHALL"**, **"SHALL NOT"**, **"SHOULD"**, **"SHOULD NOT"**, **"RECOMMENDED"**, **"NOT RECOMMENDED"**, **"MAY"**, and **"OPTIONAL"** in this document are to be interpreted as described in  
[BCP 14 / RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) and  
[RFC 8174](https://datatracker.ietf.org/doc/html/rfc8174) when written in all capitals.

**FSV File**  
: A `.fsv` archive that conforms to this specification.

**Video Format**  
: A playable video file *and its corresponding metadata entry* declared within the FSV metadata.

**Script Variant**  
: A Funscript file or logical component that defines motion or interaction data synchronized to the timeline of a referenced video. Variants may differ in axis, intensity, or style.

**Subtitle Track**  
: A subtitle or caption file referenced by the FSV metadata, such as `.srt`, `.vtt`, or `.smi`.

**Creator Metadata**  
: Information describing the original authors, sources, or contributors for each video, script, or subtitle track.

**Filestem**  
: The part of a filename before the extension. For example, the filestem of `Normal.mp4` and `Normal.funscript` is `Normal`.

---

## 3. File Layout

An `.fsv` file is a ZIP archive with a **flat directory structure**.  
All files **MUST** appear at the root of the archive, and subdirectories **MUST NOT** be used.

Video, script, and subtitle filenames shown in the examples are illustrative only;  
actual filenames are **user-defined** but **MUST exactly match** the corresponding `name` fields declared in `metadata.json`.

A valid `.fsv` archive has the following structure:

```
/
├── metadata.json
├── ExampleVideo.mp4
├── ExampleScript.funscript
├── ExampleSubtitles.srt
└── (other files referenced in the metadata)
```

Only files explicitly referenced in `metadata.json` are permitted to be present in the archive.  
Any file not declared in the metadata **MUST NOT** appear in the archive; the presence of such a file renders the container **invalid**.

Readers **MUST NOT** modify the archive implicitly, but they **MUST** detect unreferenced files and treat the container as invalid. Readers **SHOULD** warn the user or refuse to load the container until it is repaired.

Writers (including tools that create, update, or rebuild FSV archives) **MUST** remove all unreferenced files and **MUST NOT** emit an archive that contains files not declared in the metadata.

### 3.1 Required Files

The following files **MUST** be referenced in `metadata.json`.  
Their physical presence in the archive depends on whether the container is complete or incomplete (see Section 5.1).

| File | Description | Requirement |
|------|-------------|-------------|
| `metadata.json` | Describes all other contents and their relationships | **MUST** be present in the archive |
| Video files | Video entries declared in `video_formats` | **MUST** have at least one metadata entry; file **MAY** be missing in incomplete containers |
| Script files | Funscript entries declared in `script_variants` | **MUST** have at least one metadata entry; file **SHOULD** be present |

### 3.2 Optional Files

The following files **MAY** be referenced in metadata.  
If referenced, they **MUST** be included in the archive.

| File | Description |
|------|-------------|
| Additional axis scripts (`*.roll.funscript`, etc.) | Extra motion data |
| Subtitle files | Subtitle or caption files for the associated video(s) |


---

## 4. Metadata Structure

The `metadata.json` file contains all descriptive and relational information for the FSV archive.  
It **MUST** be encoded as UTF-8 without BOM. Field order within the JSON object is not significant.

### Example

```json
{
  "format_version": "1.0.0",
  "tags": ["example", "demo", "funscript", "video"],
  "title": "Example FSV Content",
  "creators": {
    "videos": [
      {
        "work_name": "Normal.mp4",
        "source_url": "https://example.com/normal_video",
        "creator_info": {
          "name": "John Doe",
          "socials": ["twitter.com/johndoe", "patreon.com/johndoe"]
        }
      }
    ],
    "scripts": [
      {
        "work_name": "Normal.funscript",
        "source_url": "https://example.com/normal_script",
        "creator_info": {
          "name": "Alice",
          "socials": ["patreon.com/alice"]
        }
      }
    ],
    "subtitles": [
      {
        "work_name": "English.srt",
        "source_url": "https://example.com/English.srt",
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
      "checksum": "sha256:abcdef..."
    }
  ],
  "script_variants": [
    {
      "name": "Normal.funscript",
      "description": "Standard script with optional roll axis",
      "additional_axes": ["roll"],
      "duration": 123456,
      "start_offset": 0,
      "checksum": "sha256:123456..."
    }
  ],
  "subtitle_tracks": [
    {
      "name": "English.srt",
      "language": "en",
      "description": "English subtitles",
      "checksum": "sha256:123456..."
    }
  ]
}
```

### 4.1 Root-Level Fields

| Field             | Type             | Description                                      | Required | Default Value               | Error Condition                                 |
|-------------------|------------------|--------------------------------------------------|----------|-----------------------------|-----------------------------------------------|
| `format_version`  | string           | Version of the FSV metadata schema.              | Yes      | *None (must be provided)*   | Missing or not a string → **Invalid container** |
| `tags`            | array of strings | Keywords or categories for discovery.            | No       | Empty array `[]`            | None                                            |
| `title` | string | Canonical, human-readable name of the content set. If omitted, readers **MAY** derive a display title from the filestem of the `.fsv` file when available. | No | None (reader **MAY** use filestem as fallback) | None |
| `creators`        | object           | Information about creators of videos, scripts, and subtitles. | No | `{ "videos": [], "scripts": [], "subtitles": [] }` | None |
| `video_formats`   | array            | Metadata entries describing referenced video files. | Yes | *None (must be provided)* | Missing or empty array → **Invalid container** |
| `script_variants` | array            | Metadata entries describing referenced Funscript files. | Yes | *None (must be provided)* | Missing or empty array → **Invalid container** |
| `subtitle_tracks` | array            | Metadata entries describing subtitle files.      | No       | Empty array `[]`            | None |

If `title` is not provided, readers **MAY** fall back to using the filestem of the `.fsv` file as a display title. This fallback is not authoritative and is only intended for cases where no explicit title is present.

If a field marked **Required** is missing or has an unexpected type, the container is in an **error state** and **MUST NOT** be treated as valid.  

Fields marked as **Optional** **MAY** be omitted; when omitted, their values **MUST** be assumed to match the **Default Value** listed above.  

Unknown additional root-level fields **MAY** appear and **SHOULD** be ignored by readers for forward compatibility. Writers **SHOULD** preserve unknown fields when rewriting metadata.

### 4.2 Creators

The `creators` object provides author and provenance information for videos, script variants, and subtitle tracks.  
This object is **OPTIONAL**; if omitted, readers **MUST** assume a default value of:

```json
{ "videos": [], "scripts": [], "subtitles": [] }
```

Each array **MAY** contain zero or more creator entries.  
If an entry is present, it **MUST** follow the structure defined below.

| Field       | Type  | Description                                                  | Required |
|-------------|-------|--------------------------------------------------------------|----------|
| `videos`    | array | Creator entries associated with video content.               | No       |
| `scripts`   | array | Creator entries associated with script or motion-data work.  | No       |
| `subtitles` | array | Creator entries associated with subtitle or caption files.   | No       |

Each creator entry has the following structure:

| Field          | Type   | Description                                                                 | Required |
|----------------|--------|-----------------------------------------------------------------------------|----------|
| `work_name`    | string | Logical identifier describing the work the creator contributed to. This value is conceptual and **does not need to match any filename or metadata field**. | Yes      |
| `source_url`   | string | Absolute URL indicating where the work originated or was published. Must be a syntactically valid absolute URL. | Yes      |
| `creator_info` | object | Metadata describing the author or contributor.                               | Yes      |

The `creator_info` object contains:

| Field        | Type             | Description                                                           | Required |
|--------------|------------------|-----------------------------------------------------------------------|----------|
| `name`       | string           | Display name or alias of the creator.                                 | Yes      |
| `socials`    | array of strings | Optional list of absolute URLs representing social or support links. Each entry **MUST** be a syntactically valid URL. | No       |

Creator entries are descriptive and non-functional.  
If a creator entry is malformed — for example, if a required field is missing, has the wrong type, or a URL is not a syntactically valid absolute URL — readers **MUST** ignore that entry and **MAY** warn the user.  
Malformed creator entries **MUST NOT** affect the validity of the container.

### 4.3 Video Formats

Each entry in the `video_formats` array describes a video file referenced by the container.  
If the corresponding video file is present in the archive, its filename **MUST** exactly match the value of the `name` field.

| Field         | Type     | Description                                                                     | Required |
|-------------- |----------|---------------------------------------------------------------------------------|----------|
| `name`        | string   | Filename of the video within the container.                                     | Yes      |
| `description` | string   | Human-readable description (e.g., "1080p version", "VR180", "Side-by-side 3D"). | No       |
| `duration`    | integer  | Duration of the video in milliseconds.                                          | No       |
| `checksum`    | string   | Hash used for integrity verification of the referenced file.                    | No       |

`duration` and `checksum` are **Optional** in the specification.  
Human authors **MAY** omit these fields.  
However, tools that generate or modify FSV containers **SHOULD** populate both fields whenever the referenced video file is available, as this improves interoperability and allows integrity verification.

A video format entry is considered **malformed** if any required field is missing or has the wrong type.  
Malformed video format entries **MUST** cause the container to be treated as **invalid**, as readers rely on this metadata for file association and synchronization.

### 4.4 Script Variants

Each entry in the `script_variants` array describes a Funscript file referenced by the container.  
If the corresponding script file is present in the archive, its filename **MUST** exactly match the value of the `name` field.

| Field             | Type     | Description                                                                                                                 | Required |
|-------------------|----------|-----------------------------------------------------------------------------------------------------------------------------|----------|
| `name`            | string   | Filename of the script within the container.                                                                               | Yes      |
| `description`     | string   | Human-readable description (e.g., "high-intensity version", "multi-axis script").                                           | No       |
| `additional_axes` | array    | List of additional axis names present in the script (e.g., `["roll"]`). This field is descriptive only and does not imply any structural requirements. | No |
| `duration`        | integer  | Duration of the script in milliseconds.                                                                                     | No       |
| `start_offset`    | integer  | Offset between the script timeline and the video timeline, in milliseconds.                                                 | No       |
| `checksum`        | string   | Hash used for integrity verification of the referenced script file.                                                         | No       |

#### Start Offset Semantics

The `start_offset` field adjusts how script playback begins relative to the video:

- **Positive values** trim the beginning of the script.  
  The script starts immediately at video time 0, but begins at script timestamp `start_offset` (in milliseconds).  
  Script actions occurring before this timestamp **MUST NOT** be played.

- **Negative values** delay script playback.  
  The script begins only after the video has played for `abs(start_offset)` milliseconds.  
  Script timestamps remain unchanged.

- **Zero** means no offset; the script begins when the video begins.

This behavior is intended so that the video start time acts as the canonical action-start point, while allowing scripts to skip intros or wait for video intros. Players and playback tools **SHOULD** honor the `start_offset` field when synchronizing a script to a video, as doing so preserves the author's intended timing.

However, players **MAY** ignore this field if their synchronization model does not use video-relative timing.  
Regardless of implementation choice, players **MUST NOT** produce out-of-bounds script behavior (e.g., negative playback timestamps or advancing beyond the available script duration).

#### Recommended Fields for Tooling

`duration`, `start_offset`, and `checksum` are **OPTIONAL** in the specification.  
Human authors **MAY** omit these fields.  
However, tools that create or modify FSV containers **SHOULD** populate `duration` and `checksum` automatically when the script file is available.

#### Validation

A script variant entry is considered **malformed** if any required field is missing or has the wrong type.  
Malformed script variant entries **MUST** cause the container to be treated as **invalid**, as readers rely on this metadata for timing, synchronization, and integrity.

### 4.5 Subtitle Tracks

The `subtitle_tracks` array defines subtitle or caption files associated with one or more videos in the container.  
Each entry in the array corresponds to a single subtitle or caption file (for example, `.srt`, `.vtt`, `.smi`).

| Field        | Type    | Description                                                   | Required |
|--------------|---------|---------------------------------------------------------------|----------|
| `name`       | string  | Filename of the subtitle file inside the container.           | Yes      |
| `language`   | string  | ISO 639-1 language code (e.g., `"en"`, `"ja"`).               | Yes      |
| `description`| string  | Human-readable label (e.g., `"English subtitles"`).           | No       |
| `checksum`   | string  | Hash used for integrity verification of the referenced file.  | No       |

If the subtitle file is present in the archive, its filename **MUST** match the `name` value exactly.  
Fields `description` and `checksum` are **OPTIONAL**, but tools that generate or rebuild FSV containers **SHOULD** populate `checksum` when the file is available, for better interoperability.

Readers **SHOULD** select subtitle tracks according to the user’s language preference.  
Readers **MAY** ignore subtitle formats they cannot support.  
Malformed subtitle entries (for example: missing `name` or `language`, wrong type) **MUST NOT** cause the container to be treated as invalid; such entries **MAY** be ignored and the reader **MAY** warn the user.

---

## 5. General Rules

- Videos referenced in `video_formats` are generally intended to depict the same scene or content.  
  Readers and tools **MAY** compare durations or metadata for consistency, but duration mismatches **MUST NOT** by themselves invalidate the container.

- Script variants in `script_variants` are intended to synchronize with the referenced videos.  
  This specification does not require scripts to match video duration exactly, and differences **MAY** be resolved through `start_offset` or player-specific synchronization behavior.

- The `work_name` field in creator entries is a logical identifier for attribution and does not need to match the filename or the `name` field of any metadata entry.  
  Tools **MAY** use it for grouping or display purposes, but readers **MUST NOT** rely on it for functional linking.

- Optional fields **MAY** be omitted entirely, but when present they **MUST NOT** be set to `null` and **MUST** conform to the expected type.

### 5.1 Incomplete or Deferred-Complete Containers

An FSV file **MAY** omit one or more video files when distribution rights or practical constraints prevent including them directly in the archive. In such cases, the container remains **metadata-valid** but is considered **content-incomplete**.

For an incomplete container:

- The `metadata.json` **MUST** still include entries for all referenced videos in `video_formats`.  
  (Creator entries associated with these videos are **OPTIONAL** and **MAY** be omitted.)

- Each omitted video **SHOULD** include a `description`, `source_url`, and `checksum` when available, so that tools and readers can identify and verify the file once obtained separately.

- Readers and tools **SHOULD** clearly indicate when referenced video files are missing, and **MUST** treat any referenced-but-missing file as content-incomplete rather than an error.

- A content-incomplete container **MUST NOT** be treated as **content-complete** or fully playable until all referenced video files are present and, when a checksum is provided, the file matches its declared digest.

- Once all referenced video files are added to the archive and their checksums (if provided) match the metadata, the container transitions to a **content-complete** state.

This mechanism allows creators to distribute FSV files with complete metadata and attribution while omitting copyrighted or restricted video files. Tools and users **MAY** later complete the container by adding the required video files.

### 5.2 Checksum Format

Checksums are used to verify the integrity of referenced files.  
When present, a checksum string **MUST** conform to the following format:

```
<algorithm_name>:<hex_encoded_digest>
```

#### 5.2.1 Algorithm Requirements

- `algorithm_name` **MUST** use lowercase ASCII letters (`a–z`), digits (`0–9`), and hyphens (`-`).
- The algorithm **MUST** represent a cryptographically secure, non-deprecated hash function at the time the checksum was generated.
- Implementations **MUST NOT** use compromised algorithms such as `md5` or `sha1`.
- The following algorithms are **RECOMMENDED** for new FSV content:
  - `sha256` (default choice for tooling)
  - `sha512`
  - `sha3-256`
- Additional algorithms **MAY** be used, provided they follow the naming rules above.

#### 5.2.2 Digest Encoding

- The digest **MUST** be encoded as a lowercase hexadecimal string without whitespace.
- The digest **MUST** represent the full output of the hash function  
  (e.g., 64 hex characters for SHA-256).
- Example valid values:
  - `sha256:3d7f7a9d2e3b0caa6abaf93a6f90b8b32a33cd12b2b4e35a3e7762c9a41a73e4`
  - `sha512:bb15f0f7d96c1d6e24ccaa47b62741ad76e79a45d5de2d72a92af7b4df1a6bb2c0ff90e2b066f8f08495d8b93b9c0a42cdb42b5ab33bc3c9d44a51a404210d91`

#### 5.2.3 Validation

A checksum string is considered **well-formed** if:

1. It matches the regular expression:

```
^[a-z0-9-]+:[a-f0-9]+$
```

2. The algorithm is not known to be deprecated (e.g., not `md5` or `sha1`).
3. If the algorithm is recognized by the reader, the digest length matches the expected output size.

If a checksum field is malformed or uses an unsupported algorithm, readers **MUST** ignore the checksum and **MAY** warn the user.  
Malformed or unsupported checksum fields **MUST NOT** invalidate the container.

---

## 6. Validation Rules

A container is considered **invalid** only if one or more of the following conditions are true:

1. `format_version` is missing, empty, not a string, or not the format as defined in section 7.

2. `video_formats` is missing, empty, or contains an entry that is malformed.  
   (Video format entries are functional; malformed entries **MUST** invalidate the container.)

3. `script_variants` is missing, empty, or contains an entry that is malformed.  
   (Script variant entries are functional; malformed entries **MUST** invalidate the container.)

4. Any field that is **required** by this specification is present but has the wrong type or an invalid value.

5. Any **functional metadata field** (e.g., filenames, durations tied to synchronization, required structural fields) is malformed in a way that prevents correct interpretation.

The following conditions **MUST NOT** invalidate the container:

- Malformed creator entries (`creators.videos`, `creators.scripts`, `creators.subtitles`); such entries **MUST** be ignored.  
- Malformed subtitle track entries; such entries **MAY** be ignored.  
- Malformed or unsupported checksum fields; such fields **MUST** be ignored.  
- Malformed optional fields that do not affect structural correctness or synchronization.

Optional fields **MAY** be omitted entirely, and omitted fields **MUST** be interpreted according to the default values defined in this specification.

---

## 7. Versioning and Compatibility

### 7.1 Format Version Rules

The `format_version` field defines the version of the FSV metadata schema used by the container.  
This field **MUST** be a string and **MUST** conform to the following strict Semantic Versioning structure:

```
MAJOR.MINOR.PATCH
```

Where:

- `MAJOR`, `MINOR`, and `PATCH` are non-negative integers.
- Leading zeroes are not permitted, except when the value is zero itself.

Examples of valid version strings include:

```
"1.0.0"
"1.2.5"
"2.10.3"
```

Pre-release identifiers and build metadata (such as `1.0.0-alpha`, `1.0.0-rc.1`, or `1.0.0+build7`) **MUST NOT** be used.  
Readers **MUST** treat any `format_version` value that does not match the `MAJOR.MINOR.PATCH` pattern as invalid.

### 7.2 Extensions

To support experimentation, prototyping, and community-driven enhancements without fragmenting the core FSV versioning model, containers **MAY** include an **OPTIONAL** `extensions` field:

```
"extensions": [ "extension-identifiers" ]
```

Rules:

- If present, the `extensions` field **MUST** be an array of strings.
- Each string **SHOULD** use a namespaced format (for example, `com.example.fsv-extra-metadata`).
- Extensions **MUST NOT** modify or override the meaning of any required field in this specification.
- Readers **MAY** ignore unrecognized or unsupported extensions.
- Malformed or unknown extensions **MUST NOT** invalidate the container.

### 7.3 Compatibility Rules

- Readers **MAY** ignore metadata fields they do not recognize.  
- Writers **SHOULD NOT** remove required fields from earlier versions without incrementing the major version.  
- Readers **SHOULD** treat containers with a higher major version than they support as potentially incompatible.  
- Minor and patch version increments **SHOULD** remain backward compatible with earlier versions under the same major version.  

These rules ensure that valid FSV containers remain interoperable across tools and future revisions of the specification.

---

## 8. Example Minimal Metadata

```json
{
  "format_version": "1.0.0",
  "video_formats": [
    { "name": "video.mp4" }
  ],
  "script_variants": [
    { "name": "Main" }
  ]
}
```
