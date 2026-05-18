# pchat — Modern Terminal Live Chat Viewer

![pchat demo](/docs/Tokio-Night.png)

pchat is a **beautiful**, **blazingly fast** terminal application to display live stream chats directly in your terminal. Built with Rust and ❤️ for the Linux community, pchat supports multiple backends (YouTube official API, mock, and experimental internal), customizable themes, and an elegant TUI that stays responsive even under heavy chat loads.

---

## ✨ Features

- **Real-time live chat** from YouTube (via official API)
- **Mock backend** for demo and testing
- **Gorgeous TUI** with double borders, rounded corners, and vibrant colors
- **7 built-in themes** + easy custom theme creation (TOML)
- **Compact / expanded modes** toggled with `c`
- **Separators** between messages for clarity
- **Stream info** header: title, channel, likes, views, watching count
- **Statistics bar**: messages per second, spam ratio, queue length, total messages
- **Async architecture** powered by Tokio – never blocks rendering
- **Low CPU / low memory** – only redraws when necessary
- **Keyboard shortcuts**: `q` / `Esc` to quit, `c` to toggle compact mode
- **Written in Rust** – safe, concurrent, and fast

---

## 📥 Installation

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- (For official backend) A YouTube Data API v3 key – see [Configuration](#-configuration)

### From source
```bash
git clone https://github.com/youruser/pchat.git
cd pchat
./build.sh
```


Or manually:

```bash

cargo build --release
./target/release/pchat --backend mock "https://example.com"
```
Install globally
```bash

./install.sh   # copies to ~/.local/bin
```

Make sure ~/.local/bin is in your PATH.

## 🚀 Quick start

Run the demo with simulated messages:

```bash

pchat --backend mock --theme cyber-neon "https://example.com"
```
To connect to a real YouTube live stream, you need a YouTube Data API key:

```bash

export YOUTUBE_API_KEY="your_api_key_here"
pchat --backend official "https://www.youtube.com/watch?v=VIDEO_ID"
```
Press q or Esc to quit. Press c to toggle compact mode.

## 🎨 Themes

pchat looks best with a theme. Select one with --theme:

```bash

pchat --theme coffee-break ...
```
Available built-in themes:

    tokyo-night-enhanced (dark, purple/blue)

    white-pearl (light, clean)

    coffee-break (warm, brown tones)

    cyber-neon (dark with neon accents)

    forest-aurora (nature-inspired)

    tokyo-night (original)

    catppuccin

    nord

Custom themes are loaded from ~/.config/pchat/themes/<name>.toml. Example:

```toml

[colors]
background = "#0d0d0d"
foreground = "#e0e0e0"
border = "#444444"
moderator = "#ff9940"
verified = "#40ff40"
member = "#4040ff"
superchat = "#ff4040"
info = "#40ffff"
alert = "#ff40ff"
```
## ⚙️ Configuration

The global config file is located at ~/.config/pchat/config.toml.

```toml

# Example config
theme = "nord"
default_backend = "mock"
compact = false
```
You can also set up themes and config with the helper script:

```bash

./setup-config.sh
```
It copies all themes to ~/.config/pchat/themes/ and creates a default config.toml.

## 🖼️ Screenshots

Cyber Neon theme
![cyber theme](/docs/Cyber.png)

Cooffee-Break theme
![cooffee theme](/docs/Coffee-Break.png)

Nord theme
![nord theme](/docs/nord.png)

## 🔧 Development

pchat is structured with modularity in mind:

    src/backend/ – Backend trait and implementations (mock, official, internal)

    src/renderer.rs – Terminal UI rendering

    src/themes.rs – Theme loader

    src/config.rs – User configuration

    scripts/ – Bash helpers

To add a new backend, implement the ChatBackend trait and register it in main.rs.

## 📄 License

MIT. See LICENSE for details.

## 🙏 Acknowledgments

Inspired by btop, lazygit, and the Rust TUI ecosystem.

Enjoy your terminal chat!
