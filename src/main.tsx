import { listen } from "@tauri-apps/api/event";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import "./App.css";

interface Widget {
  id: string;
  name: string;
  html_file: string;
  html_content: string;
  html_path?: string;
}

// state variables
let wallpaperPath: string | null = null;
let widgets: Widget[] = [];
let activeWidgets: string[] = [];
let isPlaying = true;

// base containers
const container = document.getElementById("root");
if (container) {
  container.innerHTML = `
    <main class="container">
      <div id="media-container"></div>
      <div id="widgets-layer" class="widgets-layer"></div>
    </main>
  `;
}

const mediaContainer = document.getElementById("media-container");
const widgetsLayer = document.getElementById("widgets-layer");

// parse url
const url = new URL(window.location.href);
const label = url.searchParams.get("label") || "wallpaper-0";
const idx = parseInt(label.replace("wallpaper-", ""), 10) + 1;

// video check
const isVideo = (path: string) => {
  const ext = path.split('.').pop()?.toLowerCase();
  return ["mp4", "webm", "mov"].includes(ext || "");
};

// update background media
function updateMedia() {
  if (!mediaContainer) return;
  if (!wallpaperPath) {
    mediaContainer.innerHTML = "";
    return;
  }

  const srcUrl = convertFileSrc(wallpaperPath);
  
  if (isVideo(wallpaperPath)) {
    mediaContainer.innerHTML = `
      <video
        id="wallpaper-video"
        src="${srcUrl}"
        ${isPlaying ? "autoplay" : ""}
        loop
        muted
        class="wallpaper-media"
      ></video>
    `;
    const video = document.getElementById("wallpaper-video") as HTMLVideoElement;
    if (video && !isPlaying) {
      video.pause();
    }
  } else {
    mediaContainer.innerHTML = `
      <img
        src="${srcUrl}"
        alt="Wallpaper"
        class="wallpaper-media"
        draggable="false"
      />
    `;
  }
}

// update widgets list
function updateWidgets() {
  if (!widgetsLayer) return;

  const activeList = widgets.filter(w => activeWidgets.includes(w.id));
  
  widgetsLayer.innerHTML = activeList.map(widget => {
    const src = isPlaying && widget.html_path ? convertFileSrc(widget.html_path) : "about:blank";
    const hasSrcDoc = !widget.html_path && isPlaying;
    const srcDocAttr = hasSrcDoc ? `srcdoc="${escapeHtml(widget.html_content)}"` : "";
    
    return `
      <iframe
        id="widget-iframe-${widget.id}"
        src="${src}"
        ${srcDocAttr}
        sandbox="allow-scripts"
        class="widget widget-${widget.id}"
        style="border: none; width: 100%; height: 100%; background: transparent; display: ${isPlaying ? "block" : "none"}"
        title="${widget.name}"
      ></iframe>
    `;
  }).join("");
}

// escape string
function escapeHtml(unsafe: string) {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// load initial config
async function init() {
  try {
    const path = await invoke<string>("get_monitor_wallpaper", { monitorIndex: idx });
    if (path) {
      wallpaperPath = path;
      updateMedia();
    }

    const active = await invoke<string[]>("get_monitor_widgets", { monitorIndex: idx });
    activeWidgets = active || [];

    const data = await invoke<Widget[]>("get_widgets");
    widgets = data || [];
    updateWidgets();
  } catch (err) {
    console.error("Init failed:", err);
  }
}

// listen tauri events
async function setupListeners() {
  await listen<string>(`update-monitor-${idx}`, (event) => {
    console.log("New wallpaper:", event.payload);
    wallpaperPath = event.payload;
    updateMedia();
  });

  await listen("update-widgets", async () => {
    console.log("Widgets updated, reloading...");
    try {
      const data = await invoke<Widget[]>("get_widgets");
      widgets = data || [];
      const active = await invoke<string[]>("get_monitor_widgets", { monitorIndex: idx });
      activeWidgets = active || [];
      updateWidgets();
    } catch (err) {
      console.error("Widget reload failed:", err);
    }
  });

  await listen<boolean>(`update-play-state-${idx}`, (event) => {
    console.log("Play state update:", event.payload);
    const wasPlaying = isPlaying;
    isPlaying = event.payload;
    
    const video = document.getElementById("wallpaper-video") as HTMLVideoElement;
    if (video) {
      if (isPlaying) {
        video.play().catch(err => console.error("Video play failed:", err));
      } else {
        video.pause();
      }
    }
    
    if (wasPlaying !== isPlaying) {
      updateWidgets();
    }
  });
}

// start application
init();
setupListeners();
