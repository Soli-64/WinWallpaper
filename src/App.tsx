import React, { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import "./App.css";

interface Widget {
  id: string;
  name: string;
  html_file: string;
  html_content: string;
}

// 
// Component to render widgets
// 
function WidgetComponent({ widget }: { widget: Widget }) {
  return (
    <iframe
      srcDoc={widget.html_content}
      sandbox="allow-scripts"
      className={`widget widget-${widget.id}`}
      style={{ border: "none", width: "100%", height: "100%", background: "transparent" }}
      title={widget.name}
    />
  );
}

// Only re-render when html_content changes to avoid unnecessary DOM operations
const MemoizedWidgetComponent = React.memo(WidgetComponent, (prev, next) => {
  return prev.widget.html_content === next.widget.html_content;
});

function App() {
  const [wallpaperPath, setWallpaperPath] = useState<string | null>(null);
  const [widgets, setWidgets] = useState<Widget[]>([]);
  const [activeWidgets, setActiveWidgets] = useState<string[]>([]);
  const [isPlaying, setIsPlaying] = useState<boolean>(true);
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current) {
      if (isPlaying) {
        videoRef.current.play().catch((err) => console.error("Failed to play video:", err));
      } else {
        videoRef.current.pause();
      }
    }
  }, [isPlaying, wallpaperPath]);

  const isVideo = (path: string) => {
    const ext = path.split('.').pop()?.toLowerCase();
    return ["mp4", "webm", "mov"].includes(ext || "");
  };

    useEffect(() => {
     const url = new URL(window.location.href);
     const label = url.searchParams.get("label") || "wallpaper-0";
     const idx = parseInt(label.replace("wallpaper-", ""), 10) + 1;

     invoke<string>(`get_monitor_wallpaper`, { monitorIndex: idx }).then((path) => {
       if (path) setWallpaperPath(path);
     });

     invoke<string[]>(`get_monitor_widgets`, { monitorIndex: idx }).then((active) => {
       setActiveWidgets(active || []);
     });

     invoke<Widget[]>("get_widgets")
       .then((data) => {
         setWidgets(data);
       })
       .catch((err) => console.error("Failed to load widgets:", err));

     let unlistenWallpaper: (() => void) | null = null;
     let unlistenWidgets: (() => void) | null = null;
     let unlistenPlayState: (() => void) | null = null;

     const setupListener = async () => {
        unlistenWallpaper = await listen<string>(`update-monitor-${idx}`, (event) => {
          console.log("New wallpaper:", event.payload);
          setWallpaperPath(event.payload);
          // Flush video buffer when wallpaper changes to prevent ghosting
          if (videoRef.current) {
            videoRef.current.load();
          }
        });

       unlistenWidgets = await listen("update-widgets", () => {
         console.log("Widgets updated, reloading...");
         invoke<Widget[]>("get_widgets")
           .then((data) => {
             setWidgets(data);
           })
           .catch((err) => console.error("Failed to reload widgets:", err));
         invoke<string[]>(`get_monitor_widgets`, { monitorIndex: idx }).then((active) => {
           setActiveWidgets(active || []);
         });
       });

       unlistenPlayState = await listen<boolean>(`update-play-state-${idx}`, (event) => {
         console.log("Play state update:", event.payload);
         setIsPlaying(event.payload);
       });
     };

     setupListener();

     return () => {
       if (unlistenWallpaper) unlistenWallpaper();
       if (unlistenWidgets) unlistenWidgets();
       if (unlistenPlayState) unlistenPlayState();
     };
   }, []);

  const filteredWidgets = widgets.filter(w => activeWidgets.includes(w.id));

  return (
    <main className="container">
       {wallpaperPath && (
         isVideo(wallpaperPath) ? (
           <video
             ref={videoRef}
             key={wallpaperPath}
             src={convertFileSrc(wallpaperPath)}
             autoPlay
             loop
             muted
             className="wallpaper-media"
           />
         ) : (
           <img
             key={wallpaperPath}
             src={convertFileSrc(wallpaperPath)}
             alt="Wallpaper"
             className="wallpaper-media"
             draggable={false}
           />
         )
       )}

       <div className="widgets-layer">
         {filteredWidgets.map((widget) => (
           <MemoizedWidgetComponent key={widget.id} widget={widget} />
         ))}
       </div>
    </main>
  );
}

export default App;