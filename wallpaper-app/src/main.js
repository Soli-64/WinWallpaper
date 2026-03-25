import { listen } from '@tauri-apps/api/event';

const media_type = (path) => {
    const extension = path.split('.').pop().toLowerCase();
    if (['jpg', 'jpeg', 'png'].includes(extension)) {
        return 'image';
    } else if (['mp4', 'webm'].includes(extension)) {
        return 'video';
    } else {
        return 'unknown';
    }
};

listen('wallpaper-update', (event) => {
  console.log('Wallpaper update event received:', event);
});