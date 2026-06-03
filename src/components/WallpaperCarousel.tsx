import React, { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useDragScroll } from "../hooks/useDragScroll";
import { Shuffle, Image, Film, Layers } from "lucide-react";

interface CarouselItem {
  name: string;
  path: string;
  thumb_path: string;
  is_video: boolean;
}

interface WallpaperCarouselProps {
  items: CarouselItem[];
  onWallpaperClick: (item: CarouselItem) => void;
}

export const WallpaperCarousel: React.FC<WallpaperCarouselProps> = ({ items, onWallpaperClick }) => {
  const { scrollRef, isDragging, scrollHandlers } = useDragScroll();
  const [activeFilter, setActiveFilter] = useState<"all" | "images" | "videos">("all");

  const filteredItems = items.filter((item) => {
    if (activeFilter === "images") return !item.is_video;
    if (activeFilter === "videos") return item.is_video;
    return true;
  });

  const handleShuffle = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isDragging) return;
    if (filteredItems.length === 0) return;
    const randomIndex = Math.floor(Math.random() * filteredItems.length);
    const randomItem = filteredItems[randomIndex];
    onWallpaperClick(randomItem);
  };

  return (
    <div 
      className="scroll-area" 
      ref={scrollRef} 
      {...scrollHandlers}
    >
      <div className="carousel">
        {/* Sleek Utility Control Card */}
        <div className="carousel-item control-card" onClick={(e) => e.stopPropagation()}>
          <div className="control-header">
            <Layers size={14} className="control-icon" />
            <span className="control-title">Wallpapers</span>
          </div>

          <button className="shuffle-action-btn" onClick={handleShuffle}>
            <Shuffle size={16} />
            <span>Shuffle Wallpaper</span>
          </button>

          <div className="filter-pill-container">
            <button 
              className={`filter-pill ${activeFilter === "all" ? "active" : ""}`}
              onClick={() => setActiveFilter("all")}
            >
              All
            </button>
            <button 
              className={`filter-pill ${activeFilter === "images" ? "active" : ""}`}
              onClick={() => setActiveFilter("images")}
              title="Images only"
            >
              <Image size={12} />
              <span>Images</span>
            </button>
            <button 
              className={`filter-pill ${activeFilter === "videos" ? "active" : ""}`}
              onClick={() => setActiveFilter("videos")}
              title="Videos only"
            >
              <Film size={12} />
              <span>Videos</span>
            </button>
          </div>
        </div>

        {/* Filtered Wallpapers List */}
        {filteredItems.map((item, index) => (
          <div
            key={index}
            className="carousel-item"
            onClick={(e) => {
              if (isDragging) {
                e.preventDefault();
                e.stopPropagation();
                return;
              }
              onWallpaperClick(item);
            }}
          >
            <img 
              src={convertFileSrc(item.thumb_path)} 
              alt={item.name} 
              className="thumbnail-img"
              draggable={false}
              // lazy load thumbnails
              loading="lazy"
            />
            {item.is_video && (
              <div className="video-overlay">
                <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="white" opacity="0.8">
                  <path d="M8 5v14l11-7z" />
                </svg>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
