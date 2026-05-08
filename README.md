# Windows Wallpaper Manager

A native Windows wallpaper manager built with Tauri v2 and React. It provides a seamless experience for setting both images and videos as your system wallpaper across multiple monitors.

## Features

- Multi-Monitor Support: Automatically detects all active monitors and creates dedicated background windows for each.
- Media Support: Supports standard image formats and video files (MP4, WebM).
- Automatic Thumbnails: Generates and caches thumbnails for all media types using FFmpeg.
- Widgets Support: Overlay custom HTML/CSS/JS widgets onto your desktop wallpaper with live-reloading.
- Global Shortcut: Toggle the selection interface at any time (Default: Alt + W, configurable in config.json).
- Session Persistence: Remembers and reloads your last selected wallpaper on startup.
- System Tray Integration: Right-click the tray icon for quick access to app controls, cycling wallpapers, or quitting the app.
- Performance: Built with Rust and React for a lightweight and responsive experience.

## Prerequisites

- Rust (latest stable version)
- Node.js (v18 or newer)
- FFmpeg (must be available in your system PATH for video thumbnail generation)

## Installation

1. Clone the repository.
2. Install the frontend dependencies:
   ```bash
   npm install
   ```

## Development

To run the application in development mode:
```bash
npm run tauri dev
```

## Build

To create a production build:
```bash
npm run tauri build
```

## Configuration and Storage

The application stores all data in your Documents folder under the `win-wallpaper` directory:

- Wallpapers: Place your media files in `~/Documents/win-wallpaper/wallpapers`.
- Thumbnails: Automatically generated in `~/Documents/win-wallpaper/thumbnails`.
- Widgets: Place widget HTML files in `~/Documents/win-wallpaper/widgets`.
- Settings: Last used settings and keyboard shortcut are stored in `~/Documents/win-wallpaper/config.json`.
- Widgets Configuration: Configure active widgets in `~/Documents/win-wallpaper/widgets.json`.

## Widgets

WinWallpaper supports overlaying custom widgets on top of your wallpaper. Widgets are simple HTML files that can contain CSS and JavaScript.

**How to use:**
1. Create your widget HTML file in the `~/Documents/win-wallpaper/widgets` directory. (See our [Clock Widget Example](examples/widget/clock.html) to get started!)
2. Define the widget in `~/Documents/win-wallpaper/widgets.json`:
   ```json
   [
     {
       "id": "my-clock-widget",
       "name": "Desktop Clock",
       "html_file": "clock.html"
     }
   ]
   ```
3. **Live Reloading**: WinWallpaper watches the `widgets` directory for changes. Any edits you save to your widget files will instantly reload on your desktop without needing to restart the app!

## Usage

1. Place your desired images or videos in the wallpapers directory.
2. Launch the application.
3. Use the selection bar at the bottom to switch wallpapers.
4. Press the global shortcut (Default: Alt + W) to hide or show the selection bar.

## Screenshots & Demo

![Screenshot](docs/media/screenshot_1.png)

### Demo Video

![Demo](docs/media/present_1.gif)


## Known Issues

- **Launch on Startup**: The "Launch on Startup" feature may not function correctly in some environments due to registry permission limitations or installer configuration. If the application doesn't start with Windows, activate the launch on startup option in the settings.
- **Video Thumbnails**: Video thumbnails require FFmpeg to be installed and available in your system PATH. If you just installed FFmpeg via the installer, a system restart may be required for the PATH changes to take effect.

## Contributing

Contributions are welcome! If you'd like to improve WinWallpaper, please feel free to submit a Pull Request or open an issue.

### Support the Project

If you find this project useful and would like to support its development, donations are greatly appreciated.

**BTC**: `19CdK5s3ALPcxjNxGiqM7pDZJ2AvY1SPcw` <br>
**SOL**: `9q1pTozYZRHEuYn5eMBcNGj5BvHXCRPCyzhwVhNqazN1` <br>
**ETH** (BASE): `0xDE23577a8f54E5e8EEF5eaf85438709a8178e897` <br>