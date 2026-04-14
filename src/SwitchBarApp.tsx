import { useEffect, useState } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import "./SwitchBarApp.css";

interface CarouselItem {
  name: string;
  path: string;
  thumb_path: string;
  is_video: boolean;
}

export default function SwitchBarApp() {
  const [items, setItems] = useState<CarouselItem[]>([]);

  useEffect(() => {
    invoke<CarouselItem[]>("get_wallpapers").then((data) => {
      setItems(data);
    }).catch((err) => {
      console.error("Failed to fetch wallpapers", err);
    });
  }, []);

  const handleItemClick = async (item: CarouselItem) => {
    await emit("update-wallpaper", item.path);
    await invoke("set_wallpaper_config", { path: item.path });
  };

  return (
    <div className="switch-bar-container">
      <div className="scroll-area">
        <div className="carousel">
          {items.map((item, index) => (
            <div
              key={index}
              className="carousel-item"
              onClick={() => handleItemClick(item)}
            >
              <img src={convertFileSrc(item.thumb_path)} alt={item.name} className="thumbnail-img" />
              {item.is_video && (
                <div className="video-overlay">
                  {/* Simple Play Icon */}
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="40"
                    height="40"
                    viewBox="0 0 24 24"
                    fill="white"
                    opacity="0.8"
                  >
                    <path d="M8 5v14l11-7z" />
                  </svg>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
