# Windows Wallpaper Manager

A native Windows wallpaper manager built with Tauri v2 and React. It provides a seamless experience for setting both images and videos as your system wallpaper across multiple monitors.

## Features

- Multi-Monitor Support: Automatically detects all active monitors and creates dedicated background windows for each.
- Media Support: Supports standard image formats and video files (MP4, WebM).
- Automatic Thumbnails: Generates and caches thumbnails for all media types using FFmpeg.
- Global Shortcut: Toggle the selection interface at any time using Alt + W.
- Session Persistence: Remembers and reloads your last selected wallpaper on startup.
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
- Settings: Last used settings are stored in `~/Documents/win-wallpaper/config.json`.

## Usage

1. Place your desired images or videos in the wallpapers directory.
2. Launch the application.
3. Use the selection bar at the bottom to switch wallpapers.
4. Press Alt + W to hide or show the selection bar.

## Contributing

Contributions are welcome! If you'd like to improve WinWallpaper, please feel free to submit a Pull Request or open an issue.

### Support the Project

If you find this project useful and would like to support its development, donations are greatly appreciated.

**BTC**: `19CdK5s3ALPcxjNxGiqM7pDZJ2AvY1SPcw` <br>
**SOL**: `9q1pTozYZRHEuYn5eMBcNGj5BvHXCRPCyzhwVhNqazN1` <br>
**ETH** (BASE): `0xDE23577a8f54E5e8EEF5eaf85438709a8178e897` <br>