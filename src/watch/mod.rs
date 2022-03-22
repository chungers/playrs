
use clap::{Args as clapArgs};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::fmt;


#[derive(clapArgs)]
pub struct Args {

    /// The filesystem path to watch
    path: Option<String>,

    /// True to watch recursively from given path
    #[clap(short)]
    recursive: bool,
}

impl fmt::Debug for Args {
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Arg")
        .field("path", &self.path)
        .field("recursive", &self.recursive)
        .finish()
    }
}

pub fn watch(args: &Args) {
    println!("watch was used with arg: {:?}", args);

    futures::executor::block_on(async {
        if let Err(e) = async_watch(args.path.as_ref().unwrap()).await {
            println!("error: {:?}", e)
        }
    });
}


fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            tx.send(res).await.unwrap();
        })
    })?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
