# Loss

![](https://img.shields.io/badge/loss-is%20more%20%3A%29-9cf)

Loss is a modern terminal pager and log viewer designed for efficient log file viewing and navigation.

## Features

- Fast log file viewing with efficient memory usage
- Real-time log following
- Search/Jump support
- Timestamp-based navigation
- Bookmark support
- Finder with multiple slots

## Installation

Several alternatives:

### Install from crates.io
```bash
cargo install loss-viewer
```

### Download from github release test
Download binary from [Release](https://github.com/Gusabary/Loss/releases) page and put it into $PATH

### Build from source
```bash
# clone the repository
git clone https://github.com/yourusername/loss.git
cd loss

# build the project
cargo build --release

# install the binary
cargo install --path .
```

## Usage

To view a log file:
```bash
loss <filename>
```

## Key Bindings
| Category | Key | Description |
|----------|-----|-------------|
| Basic | `q` | Exit |
| | `w` | Toggle wrap line |
| | `F` | Enter follow mode |
| | `h` | Toggle helper menu |
| Search | `/` | Search down |
| | `?` | Search up |
| | `n` | Search next |
| | `N` | Search previous |
| Jump | `t` | Jump to timestamp |
| | `j` | Jump down n lines |
| | `J` | Jump up n lines |
| | `PageUp/Down` | Jump up/down 5 lines |
| | `Ctrl+PageUp/Down` | Jump up/down 20 lines |
| | `Home` | Jump to start |
| | `End` | Jump to end |
| | `,` | Undo window vertical move |
| | `.` | Redo window vertical move |
| Bookmark | `b` | Set bookmark |
| | `g` | Open bookmark menu |
| Finder | `+` | Add active slot |
| | `-` | Remove active slot |
| | `0-9` | Switch active slot |
| | `o` | Toggle highlight flag |
| | `r` | Toggle raw/regex pattern |
| | `x` | Clear slot content |
| | `m` | Open finder menu |

## License

[MIT](https://github.com/Gusabary/Loss/blob/master/LICENSE)
