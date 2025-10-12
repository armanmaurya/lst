# nva

A fast, colorful CLI tool for listing directories, inspired by `tree` and designed for developers who live in the terminal.

## Features
- Prints directory trees with indentation and Unicode symbols
- Colors directories blue and files green for easy distinction (only in terminal output)
- Supports filtering hidden files and directories (like `.git`)
- Adjustable max depth for traversal (`--depth` or `-d`)
- Option to show or hide hidden files/directories with `-a`/`--all`
- Fast and cross-platform (Rust)
- Search for files or directories by name with `--find <PATTERN>`
- Export tree output to a file (plain text, no color)
- Print file contents with syntax highlighting for supported languages
- Unified output logic: same tree format for terminal and file

## Installation

### Quick Install (Windows)

```powershell
iwr -useb https://raw.githubusercontent.com/armanmaurya/nva/main/scripts/install.ps1 | iex
```

### From Source

```
cargo install --path .
```

Or clone and build manually:

```
git clone https://github.com/armanmaurya/nva.git
cd nva
cargo build --release
```

### Windows (Download latest release automatically)

#### Remote Installation (Recommended)

Run the installer directly from GitHub without cloning the repository:

```powershell
iwr -useb https://raw.githubusercontent.com/armanmaurya/nva/main/scripts/install.ps1 | iex
```

Or with options:

```powershell
& ([scriptblock]::Create((iwr -useb https://raw.githubusercontent.com/armanmaurya/nva/main/scripts/install.ps1))) -Force
```

#### Local Installation

If you have already cloned the repository, you can run the installer locally:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1
```

#### Options:
- Override install directory: `-InstallDir "$env:LOCALAPPDATA\nva\bin"`
- Force overwrite existing: `-Force`

### Uninstall (Windows)

```
powershell -ExecutionPolicy Bypass -File .\scripts\uninstall.ps1
```

Options:
- Remove from PATH: `-RemoveFromPath`
- Skip confirmations: `-Force`
- Remove empty install folder(s): `-Purge`

## Usage

```
nva <PATH|FILE> [OPTIONS]
```

- If `<PATH>` is a directory, prints the directory tree.
- If `<FILE>` is a file, prints its contents with syntax highlighting (if supported).
- If no argument is given, lists the current directory.

### Options

- `-a, --all` : Show hidden files and directories
- `-d, --depth <DEPTH>` : Max depth of traversal (default: 1, use 0 for unlimited)
- `--find <PATTERN>` : Search for files or directories by name (case-insensitive)
- `-o, --output <FILE>` : Export the tree to a file (plain text, no color)
- `--reverse` : Output in reverse order

### Examples

List the directory tree up to 2 levels deep:
```
nva . -d 2
```

Output:
```
.
├─ Cargo.toml
├─ README.md
├─ src/
│  ├─ main.rs
└─ target/
```

Search for all files and directories containing "main":
```
nva . --find main
```

Export the tree to a file (no color):
```
nva . --output tree.txt
```

Print a file with syntax highlighting:
```
nva src/main.rs
```

## License

MIT