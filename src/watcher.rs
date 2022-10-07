use notify::event::CreateKind;
use notify::event::DataChange;
use notify::event::ModifyKind;
use notify::EventKind::{Create, Modify, Remove};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use ws;

fn refresh(url: String) -> ws::Result<()> {
    ws::connect(format!("ws://{}/ws", url), |out| {
        out.send("refresh!").unwrap();

        move |msg| {
            println!("Client got message '{}'. ", msg);
            out.close(ws::CloseCode::Normal)
        }
    })
    .unwrap();
    Ok(())
}

fn refresh_by_event(event: notify::Event, url: String) {
    match event.kind {
        Create(CreateKind::File) => Some(refresh(url)),
        Modify(ModifyKind::Data(DataChange::Content)) => Some(refresh(url)),
        Remove(_) => Some(refresh(url)),
        _ => Some(Ok(())),
    };
    println!("changed: {:?}", event);
}

// based on https://github.com/notify-rs/notify/blob/notify-5.0.0/examples/monitor_raw.rs
pub fn watch<P: AsRef<Path>>(path: P, url: String) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    for res in rx {
        let client_url = url.clone();
        match res {
            Ok(event) => refresh_by_event(event, client_url),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
