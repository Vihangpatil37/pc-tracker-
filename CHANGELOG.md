# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - Initial Release

### Added
- **Core Tracking Engine**: Win32 API based background collector for precise window polling and idle detection.
- **Local Database Engine**: High-performance asynchronous SQLite integration for storing millions of sessions locally.
- **Modern UI**: React and TailwindCSS based Dashboard featuring:
  - Aggregate statistics (Total Time, Average Session Time).
  - Chronological activity timeline.
  - Top applications leaderboard.
- **Idle Detection**: Automatic handling of idle time (no keyboard or mouse movement) to ensure accurate active screen time.
- **Privacy Guarantee**: 100% local architecture. No network requests are made by the application for data collection.
- **Zero-Config Installer**: One-click `.exe` installer.
