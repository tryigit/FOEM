# Security Policy

## Supported Versions

| Version | Supported |
| :--- | :--- |
| 0.1.x | Yes |

## Reporting a Vulnerability

If you discover a security vulnerability in FOEM, please report it responsibly.

**Do not open a public issue for security vulnerabilities.**

Instead, contact the maintainers directly:

<p>
  <a href="https://t.me/tryigitx">
    <img src="https://img.shields.io/badge/Report-@tryigitx-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white" alt="Report via Telegram">
  </a>
</p>

Please include:

- A description of the vulnerability and its potential impact
- Steps to reproduce the issue
- Any relevant logs or screenshots

We aim to respond within 48 hours and will work with you to resolve the issue before any public disclosure.

## Device Safety Notice

FOEM interacts with low-level device hardware including bootloaders, EDL modes, IMEI data, EFS partitions, and baseband processors. Incorrect use of these features can cause permanent device damage. Always ensure you understand the risks before performing any operation. See the LICENSE file for the full disclaimer.

## Code Security

- All dependencies are checked against the GitHub Advisory Database before inclusion.
- The application does not collect, transmit, or store any personal data.
- All device communication happens locally via ADB and Fastboot over USB.
- No telemetry, analytics, or remote code execution is included.
