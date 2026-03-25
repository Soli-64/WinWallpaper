
mod ui;
mod services;

use ui::{run_app};
use services::{
   storage::ensure_storage_initialized,
   thumbnail::ThumbnailManager
};

fn main() {
   
   ensure_storage_initialized();

   std::thread::spawn(move || {
      let th_man = ThumbnailManager::new();
      th_man.cleanup_orphaned_thumbnails();
      th_man.generate_fast_thumbs();
   });
   
   let _ = run_app();
   
}
