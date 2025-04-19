# Loss

![](https://img.shields.io/badge/loss-v0.1.0-9cf)

Loss is a modern terminal pager and log viewer designed for efficient log file viewing and navigation. 

## Features

- Fast log file viewing with efficient memory usage
- Real-time log following
- Search/Jump support
- Timestamp-based navigation
- Bookmark support
- Finder with multiple slots

## Installation

### Building from Source

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

### Basic
- `q`: Exit
- `w`: Toggle wrap line
- `F`: Enter follow mode
- `h`: Toggle helper menu

### Search
- `/`: Search down
- `?`: Search up
- `n`: Search next
- `N`: Search previous

### Jump
- `t`: Jump to timestamp
- `j`: Jump down n lines
- `J`: Jump up n lines
- `PageUp/Down`: Move 5 lines
- `Ctrl + PageUp/Down`: Move 20 lines
- `Home`: Go to start
- `End`: Go to end
- `,`: Undo window vertical move
- `.`: Redo window vertical move

### Bookmark
- `b`: Set bookmark
- `g`: Open bookmark menu

### Finder
- `+`: Add active slot
- `-`: Remove active slot
- `0-9`: Switch active slot
- `o`: Toggle highlight flag
- `r`: Toggle raw/regex pattern
- `x`: Clear slot content 
- `m`: Open finder menu

## License

[MIT](https://github.com/Gusabary/Loss/blob/master/LICENSE)
