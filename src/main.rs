
// cargo add tokio -F macros,rt-multi-thread,signal,time
// cargo add tokio-util -F rt
use tokio_util::sync::CancellationToken;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Shared {
    counter: u32,
    return_code: i32,
}

async fn write_data(shared: Arc<Mutex<Shared>>, token: CancellationToken) {

    // Shall not cause data corruption
    loop {
        // needs to be protected by signal masking
        {
            println!("START writing data");
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            {
                let mut data = shared.lock().unwrap();
                data.counter += 1;
            }
            println!("END writing data");
        }

        if token.is_cancelled() {
            // The token was cancelled, task can shut down
            println!("WRITING CANCELED");
            return;
        }

        // add possible condition to set return code on failed action
        if false {
            {
                let mut data = shared.lock().unwrap();
                data.return_code = 1;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

/* copied from this post: https://stackoverflow.com/a/77591939 */

#[cfg(unix)]
async fn wait_for_shutdown(token: CancellationToken) -> i32 {
    use tokio::signal::unix::{signal, SignalKind};

    // Infos here:
    // https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = signal_interrupt.recv() => {
            println!("Received Ctrl+C");
            token.cancel();
            1
        },
        _ = signal_terminate.recv() => {
            println!("Received SIGTERM.");
            token.cancel();
            1
        },
        _ = token.cancelled() => {println!("Received Cancellation Token"); 0 }
    }
}

#[cfg(windows)]
async fn wait_for_shutdown(token: CancellationToken) -> i32 {
    use tokio::signal::windows;

    // Infos here:
    // https://learn.microsoft.com/en-us/windows/console/handlerroutine
    let mut signal_c = windows::ctrl_c().unwrap();
    let mut signal_break = windows::ctrl_break().unwrap();
    let mut signal_close = windows::ctrl_close().unwrap();
    let mut signal_shutdown = windows::ctrl_shutdown().unwrap();

    tokio::select! {
        _ = signal_c.recv() => { println!("Received CTRL_C."); 1 },
        _ = signal_break.recv() => { println!("Received CTRL_BREAK."); 1 },
        _ = signal_close.recv() => { println!("Received CTRL_CLOSE."); 1 },
        _ = signal_shutdown.recv() => { println!("Received CTRL_SHUTDOWN."); 1 },
        _ = token.cancelled() => { println!("Received Cancellation Token"); 0 },
    }
}

#[tokio::main]
async fn main() {

    // adds token to broadcast cancellation
    let token = CancellationToken::new();

    // add tracker to shut down gracefully
    let tracker = tokio_util::task::TaskTracker::new();

    // initialize shared object
    let shared = Arc::new(Mutex::new(Shared::default()));

    // start
    println!("Starting");

    // "create" temp file
    println!("Creating temp file");

    // async function writing data
    let write_shared_copy = shared.clone();
    let write_token = token.clone();
    tracker.spawn(async move {
        write_data(write_shared_copy, write_token.clone()).await;
        if !write_token.is_cancelled() {
            println!("send cancel");
            write_token.cancel();
            println!("after cancel");
        }
    });

    // async function polling data
    let polling_token = token.clone();
    tracker.spawn(async move {
        let mut counter: i32 = 0;
        let max_read = 30;
        println!("Start polling worker");
        loop {
            println!("  Polling start");
            tokio::select! {
                // Step 3: Using cloned token to listen to cancellation requests
                _ = polling_token.cancelled() => {
                    // The token was cancelled, task can shut down
                    println!("  POLLING CANCELED");
                    return;
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    // Long work has completed
                    println!("  Polling sleep finished");
                }
            }
            counter += 1;

            println!("  Polling #{counter} done");

            if counter >= max_read {
                println!("Finished Reading");
                polling_token.cancel();
                return;
            }
        }
    });

    tracker.close();

    // intermediate stuff to do
    println!("Hello, world!");

    // handle all signals and graceful shutdown
    let mut return_code = wait_for_shutdown(token.clone()).await;

    println!("Shutdown initiated");

    // handle double Ctrl+C by user
    // force exit if shutdown process keeps hanging
    tokio::spawn(async move {
        #[cfg(unix)]
        let mut signal_interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();
        #[cfg(windows)]
        let mut signal_interrupt = tokio::signal::windows::ctrl_c().unwrap();

        signal_interrupt.recv().await;
        println!("Double Ctrl+C");
        std::process::exit(1);
    });

    // wait for all tasks to finish
    tracker.wait().await;

    // cleanup the temp file
    println!("Cleaning up temp file");

    // print data from shared object
    println!("Result: {shared:#?}");

    // check if task returned an issue on regular shutdown
    if return_code == 0 {
        let data = shared.lock().unwrap();
        return_code = data.return_code;
    }

    // exit
    std::process::exit(return_code);
}
