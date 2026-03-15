# FOEM Translation Guide

This document provides a complete, step-by-step guide for contributors who wish to translate FOEM into a new language. The localization system is built on a straightforward JSON-based architecture that requires no programming experience to contribute to.

---

## Table of Contents

1. [Localization Architecture](#localization-architecture)
2. [Prerequisites](#prerequisites)
3. [Step-by-Step: Adding a New Language](#step-by-step-adding-a-new-language)
4. [Translation Rules and Conventions](#translation-rules-and-conventions)
5. [Testing Your Translation](#testing-your-translation)
6. [Submitting Your Translation](#submitting-your-translation)
7. [Maintaining Translations](#maintaining-translations)
8. [Locale File Reference](#locale-file-reference)

---

## Localization Architecture

FOEM uses a dynamic internationalization (i18n) system based on JSON locale files. Each supported language is represented by a single JSON file stored in the `locales/` directory at the project root.

```
FOEM/
  locales/
    en.json        <-- English (reference / source of truth)
    tr.json        <-- Turkish (example)
    de.json        <-- German (example)
    fr.json        <-- French (example)
```

The English locale file (`en.json`) is the canonical reference. All other locale files must mirror its structure exactly, replacing only the string values with their translated equivalents.

At runtime, the application loads the appropriate locale file based on the user's language preference in the Settings panel. If a translation key is missing from the active locale, the application falls back to the English string.

---

## Prerequisites

- A text editor that supports UTF-8 encoding (VS Code, Sublime Text, Notepad++, or any equivalent).
- A basic understanding of JSON syntax (key-value pairs, nested objects, string quoting).
- Familiarity with the language you intend to translate into.
- A GitHub account (for submitting contributions via pull request).

---

## Step-by-Step: Adding a New Language

### Step 1: Fork and Clone the Repository

Fork the FOEM repository on GitHub, then clone your fork locally:

```
git clone https://github.com/<your-username>/FOEM.git
cd FOEM
```

### Step 2: Create a New Branch

Create a dedicated branch for your translation work:

```
git checkout -b localization/<language-code>
```

Use the ISO 639-1 two-letter language code (e.g., `tr` for Turkish, `de` for German, `fr` for French, `ja` for Japanese, `ar` for Arabic).

### Step 3: Copy the English Locale File

Copy `locales/en.json` to a new file named with your language code:

```
cp locales/en.json locales/<language-code>.json
```

For example, to create a Turkish translation:

```
cp locales/en.json locales/tr.json
```

### Step 4: Update the Metadata Block

Open your new locale file and update the `meta` section at the top:

```json
{
  "meta": {
    "language": "Turkish",
    "locale_code": "tr",
    "direction": "ltr",
    "version": "1.0.0",
    "author": "Your Name or GitHub Username"
  }
}
```

Field descriptions:

| Field         | Description                                                         |
| :------------ | :------------------------------------------------------------------ |
| `language`    | The full name of the language in English (e.g., "Turkish", "German") |
| `locale_code` | ISO 639-1 two-letter code (e.g., "tr", "de", "fr")                  |
| `direction`   | Text direction: `"ltr"` (left-to-right) or `"rtl"` (right-to-left)  |
| `version`     | Version of the translation, starting at `"1.0.0"`                    |
| `author`      | Your name or GitHub username for attribution                         |

### Step 5: Translate the String Values

Translate every string value in the file. Do NOT modify the JSON keys (the left side of the colon). Only modify the values (the right side of the colon).

Correct:

```json
"title": "Cihaz Onarimi"
```

Incorrect (key was changed):

```json
"baslik": "Cihaz Onarimi"
```

Work through each section methodically:
- `app` -- General application strings (buttons, labels, common words)
- `sidebar` -- Navigation panel labels
- `device` -- Device information strings
- `bootloader` -- Bootloader management strings
- `repair` -- All repair module strings (IMEI, GMS, EFS, NV, DRK, baseband, CSC, Knox)
- `network` -- Network and security module strings (FRP, carrier, MDM, Knox)
- `flash` -- Flashing tools strings (EDL, fastboot, recovery, firmware, MTK, partitions, root)
- `hardware` -- Hardware diagnostics strings
- `tools` -- Utility tools strings
- `settings` -- Settings panel strings
- `status` -- Status and progress messages
- `errors` -- Error messages
- `dialogs` -- Confirmation dialog messages

### Step 6: Validate JSON Syntax

Before committing, validate that your file is well-formed JSON. You can use any of these methods:

Using Python:

```
python3 -c "import json; json.load(open('locales/<language-code>.json'))"
```

Using Node.js:

```
node -e "require('./locales/<language-code>.json')"
```

Using an online validator such as jsonlint.com.

If the command completes without error, the JSON is valid.

### Step 7: Commit and Push

```
git add locales/<language-code>.json
git commit -m "Add <Language> localization (<language-code>.json)"
git push origin localization/<language-code>
```

### Step 8: Open a Pull Request

Navigate to the FOEM repository on GitHub and open a pull request from your branch. In the PR description, include:

- The language name and locale code.
- Whether the translation is complete or partial.
- Any notes on regional dialect choices or terminology decisions.

---

## Translation Rules and Conventions

1. **Do not modify JSON keys.** Only translate the string values.

2. **Preserve all placeholders.** If a string contains placeholders such as `{device_name}` or `{version}`, keep them exactly as they appear. These are replaced programmatically at runtime.

3. **Do not translate technical terms that are universally recognized.** Terms like "ADB", "Fastboot", "EDL", "BROM", "EFS", "IMEI", "QCN", "NV", "GPT", "CRC", "USB", "APK", "vbmeta", "Magisk", "TWRP", and "Odin" should remain in English.

4. **Preserve JSON structure.** Do not add, remove, or reorder keys. The structure must remain identical to `en.json`.

5. **Use UTF-8 encoding.** Save all locale files with UTF-8 encoding without BOM.

6. **Keep translations concise.** UI labels need to fit within the interface layout. Avoid excessively long translations where a shorter equivalent exists.

7. **Maintain formal tone.** FOEM is a technical tool. Use formal or neutral register appropriate for software interfaces in the target language.

8. **Right-to-left languages.** If your language is written right-to-left (Arabic, Hebrew, Persian, Urdu), set the `direction` field to `"rtl"` in the metadata block. The application will adjust layout rendering accordingly.

---

## Testing Your Translation

To verify your translation renders correctly in the application:

1. Build and run FOEM from source:

```
cargo run --release
```

2. Open the Settings panel and change the language to your new locale.

3. Navigate through every panel and verify:
   - All strings are translated (no English fallback text appearing unexpectedly).
   - No strings are truncated or overflow their UI containers.
   - Placeholder substitutions render correctly.
   - Right-to-left layout is correct (if applicable).

4. Check the application log for any missing key warnings, which indicate keys present in `en.json` but absent from your locale file.

---

## Submitting Your Translation

All translations are submitted via GitHub pull requests. The review process:

1. A maintainer will review the PR for structural correctness (valid JSON, all keys present, no key modifications).
2. If possible, a native speaker among the maintainers or community will review translation quality.
3. Once approved, the translation is merged and becomes available in the next release.

Partial translations are accepted. Any missing keys will fall back to the English string at runtime. However, complete translations are strongly preferred.

---

## Maintaining Translations

When new features are added to FOEM, new keys may be added to `en.json`. When this happens:

1. A GitHub issue will be opened listing the new keys that require translation.
2. Existing translators are encouraged to update their locale files with the new keys.
3. Until updated, new keys will display in English as a fallback.

To check if your locale file is missing keys compared to `en.json`, you can use the following approach:

```
python3 -c "
import json, sys

with open('locales/en.json') as f:
    en = json.load(f)
with open('locales/<language-code>.json') as f:
    target = json.load(f)

def find_missing(ref, tgt, path=''):
    for key in ref:
        current = path + '.' + key if path else key
        if key not in tgt:
            print(f'Missing: {current}')
        elif isinstance(ref[key], dict) and isinstance(tgt.get(key), dict):
            find_missing(ref[key], tgt[key], current)

find_missing(en, target)
"
```

---

## Locale File Reference

The following table lists the top-level sections in `en.json` and their purpose:

| Section      | Description                                         |
| :----------- | :-------------------------------------------------- |
| `meta`       | Locale metadata (language name, code, direction)     |
| `app`        | General application strings (buttons, labels)        |
| `sidebar`    | Navigation sidebar labels                            |
| `device`     | Device detection and info strings                    |
| `bootloader` | Bootloader management module strings                 |
| `repair`     | Device repair module strings (all sub-modules)       |
| `network`    | Network and security module strings                  |
| `flash`      | Flashing tools module strings (all protocols)        |
| `hardware`   | Hardware diagnostics module strings                  |
| `tools`      | Utility tools module strings                         |
| `settings`   | Settings panel strings                               |
| `status`     | Operation status and progress messages               |
| `errors`     | Error messages displayed to the user                 |
| `dialogs`    | Confirmation and prompt dialog messages              |

---

For questions about the localization system or translation conventions, open an issue on the GitHub repository or contact the maintainers via the channels listed in the README.
