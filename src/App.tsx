import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [wallpaperPath, setWallpaperPath] = useState<string | null>(null);

  const isVideo = (path: string) => {
    const ext = path.split('.').pop()?.toLowerCase();
    return ["mp4", "webm", "mov"].includes(ext || "");
  };

  useEffect(() => {
    invoke<string>("get_default_wallpaper").then((path) => {
      if (path) setWallpaperPath(path);
    });

    const setupListener = async () => {
      const unlisten = await listen<string>("update-wallpaper", (event) => {
        console.log("New wallpaper:", event.payload);
        setWallpaperPath(event.payload);
      });
      return unlisten;
    };

    const promise = setupListener();
    return () => {
      promise.then(unlisten => unlisten());
    };
  }, []);

  return (
    <main className="container">
      {wallpaperPath && (
        isVideo(wallpaperPath) ? (
          <video
            src={convertFileSrc(wallpaperPath)}
            autoPlay
            loop
            muted
            className="wallpaper-media"
          />
        ) : (
          <img
            src={convertFileSrc(wallpaperPath)}
            alt="Wallpaper"
            className="wallpaper-media"
          />
        )
      )}
    </main>
  );
}

export default App;
