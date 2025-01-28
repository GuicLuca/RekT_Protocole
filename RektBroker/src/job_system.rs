use std::sync::atomic::Ordering;

use tokio::task::JoinHandle;
use tokio::{join, task};

use crate::prelude::Result;
use crate::{PACKET_BUFFER, SERVER_IS_RUNNING, WORKER_CONDVAR};

pub async fn init_job_system() -> Result<()> {
    let num_cores = num_cpus::get(); // Get the number of physical cores

    let mut workers: Vec<JoinHandle<()>> = Vec::with_capacity(num_cores);

    for _ in 0..num_cores {
        workers.push(task::spawn_blocking(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(js_worker());
        }));
    }

    join!(async {
        for handle in workers {
            handle.await;
        }
    });

    Ok(())
}

///
/// js_worker are started in `init_job_system` method.
/// Each worker get packet to compute from the PACKET_BUFFER and
/// try to empty it while the server is running.
///
///
async fn js_worker() {
    while SERVER_IS_RUNNING.load(Ordering::Acquire) {
        // Get the first packet of the queue
        let packet = match PACKET_BUFFER.pop() {
            Some(packet) => packet,
            None => {
                // If no packet : lock the thread until there is some packet to compute
                let &(ref lock, ref cvar) = &*WORKER_CONDVAR.clone();
                let mut waiting = lock.lock();
                if PACKET_BUFFER.is_empty() {
                    // buffer is empty : wait on this line until the condvar is notified
                    cvar.wait(&mut waiting);
                }
                continue;
            }
        };

        // compute the packet
        crate::handle_datagram(packet).await;
    }
}
