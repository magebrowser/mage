# Rust Browser with WebView2

A simple web browser built in Rust using Microsoft's WebView2 for Windows.

## Features

- ✅ Full web browsing capabilities using WebView2 (Microsoft Edge rendering engine)
- ✅ URL bar with automatic HTTPS protocol addition
- ✅ Navigation buttons (Back, Forward)
- ✅ Refresh button
- ✅ Go button to navigate to entered URLs
- ✅ Automatic URL bar updates when navigating

## Prerequisites

- **Windows 10/11** (WebView2 is Windows-only)
- **Rust** (install from [rustup.rs](https://rustup.rs/))
- **WebView2 Runtime** (usually pre-installed on Windows 11, or download from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/))

## Building

1. Clone or download this project
2. Open a terminal in the project directory
3. Build the project:

```bash
cargo build --release
```

## Running

After building, run the browser:

```bash
cargo run --release
```

Or run the executable directly:

```bash
target/release/rust-browser.exe
```

## Usage

- **Enter a URL**: Type in the URL bar and click "Go" or press Enter
- **Navigate Back**: Click the "← Back" button
- **Navigate Forward**: Click the "Forward →" button  
- **Refresh Page**: Click the "⟳ Refresh" button
- **Browse**: The browser will automatically add "https://" if you don't include a protocol

## Technical Details

- **WebView2**: Uses Microsoft Edge's Chromium-based rendering engine (but not bundled Chromium)
- **Native Windows UI**: Uses Win32 API for window creation and controls
- **Async WebView Initialization**: WebView2 is initialized asynchronously for better performance

## Notes

- The browser starts at Google's homepage by default
- URLs without a protocol (http:// or https://) will automatically get https:// prepended
- The URL bar updates automatically as you navigate through pages
