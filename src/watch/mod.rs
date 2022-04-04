
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use signal_hook::{
    consts::SIGINT,
    consts::SIGTERM,
    iterator::Signals,
};
use std::{path::Path, process, thread};


pub mod command;
use command::Verb::{Run, Stop};


pub fn watch(cmd: &command::Command) {
    println!("watch was used with arg: {:?}", cmd);

    match cmd.verb.as_ref().unwrap() {
        Run(args) => {
            println!("Runing {:?}", args);


            let mut signals = match Signals::new(&[SIGINT, SIGTERM]) {
                Err(err) => {
                    println!("Error {:?}", err);
                    return;
                },
                Ok(s) => s,
            };

            thread::spawn(move || {
                for sig in signals.forever() {
                    println!("Received signal {:?}", sig);

                    match sig {
                        SIGINT => { println!("SIGINT"); process::exit(0x0100); },
                        SIGTERM => { println!("SIGTERM"); process::exit(0x0100); },
                        _ => println!("Others {:?}", sig),
                    }
                }
            });


            futures::executor::block_on(async {
                if let Err(e) = async_watch(args.path.as_ref().unwrap()).await {
                    println!("error: {:?}", e)
                }
            });
        },
        Stop(args) => {
            println!("Stopping: {:?}", args);
        },
    }
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
