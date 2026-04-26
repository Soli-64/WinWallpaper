import { useEffect, useState, useRef } from "react";
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
  const scrollRef = useRef<HTMLDivElement>(null);

  const [isDown, setIsDown] = useState(false);
  const [startX, setStartX] = useState(0);
  const [scrollLeftState, setScrollLeftState] = useState(0);
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    // Force focus on mount
    window.focus();
    
    const handleKeyDown = (e: KeyboardEvent) => {
      if (scrollRef.current) {
        if (e.key === 'ArrowRight') {
          scrollRef.current.scrollBy({ left: 300, behavior: 'smooth' });
        } else if (e.key === 'ArrowLeft') {
          scrollRef.current.scrollBy({ left: -300, behavior: 'smooth' });
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);

    invoke<CarouselItem[]>("get_wallpapers").then((data) => {
      setItems(data);
    }).catch((err) => {
      console.error("Failed to fetch wallpapers", err);
    });

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  const handleItemClick = async (item: CarouselItem) => {
    await emit("update-wallpaper", item.path);
    await invoke("set_wallpaper_config", { path: item.path });
  };

  const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    if (scrollRef.current) {
      scrollRef.current.scrollLeft += e.deltaY;
    }
  };

  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    setIsDown(true);
    setIsDragging(false);
    if (scrollRef.current) {
      setStartX(e.pageX - scrollRef.current.offsetLeft);
      setScrollLeftState(scrollRef.current.scrollLeft);
    }
  };

  const handleMouseLeave = () => {
    setIsDown(false);
  };

  const handleMouseUp = () => {
    setIsDown(false);
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!isDown) return;
    e.preventDefault();
    if (scrollRef.current) {
      const x = e.pageX - scrollRef.current.offsetLeft;
      const walk = (x - startX) * 1.5; // Scroll speed multiplier
      scrollRef.current.scrollLeft = scrollLeftState - walk;
      
      if (Math.abs(walk) > 5) {
        setIsDragging(true);
      }
    }
  };

  return (
    <div className="switch-bar-container">
      <div 
        className="scroll-area" 
        ref={scrollRef} 
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseLeave={handleMouseLeave}
        onMouseUp={handleMouseUp}
        onMouseMove={handleMouseMove}
      >
        <div className="carousel">
          {items.map((item, index) => (
            <div
              key={index}
              className="carousel-item"
              onClick={(e) => {
                if (isDragging) {
                  e.preventDefault();
                  e.stopPropagation();
                  return;
                }
                handleItemClick(item);
              }}
            >
              <img 
                src={convertFileSrc(item.thumb_path)} 
                alt={item.name} 
                className="thumbnail-img"
                draggable={false}
              />
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
