use std::{path::Path, sync::mpsc::channel, time::Duration};

use log::{error, trace};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

pub fn watch_directories<F, P>(
    templates_dir: P,
    posts_dir: P,
    static_dir: P,
    listener: F,
) -> notify::Result<()>
where
    P: AsRef<Path>,
    F: FnOnce(notify::DebouncedEvent) -> () + Copy,
{
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;

    (watcher.watch(templates_dir, RecursiveMode::Recursive))?;
    (watcher.watch(static_dir, RecursiveMode::Recursive))?;
    (watcher.watch(posts_dir, RecursiveMode::Recursive))?;

    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(..)
                | DebouncedEvent::Create(..)
                | DebouncedEvent::Remove(..)
                | DebouncedEvent::Rename(..) => listener(event),

                _ => trace!("ignoring change: {:#?}", event),
            },
            Err(e) => error!("watch error: {:?}", e),
        }
    }
}
