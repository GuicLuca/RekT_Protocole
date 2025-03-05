use rekt_lib::datagrams::heartbeat_requests::DtgHeartbeatRequest;
use rekt_lib::enums::connection_status::ConnectionStatus;
use rekt_lib::enums::connection_status::ConnectionStatus::*;
use rekt_lib::enums::datagram_type::DatagramType::HeartbeatRequest;
use std::sync::atomic::Ordering;
use tokio::task::JoinHandle;
use tokio::{join, task};

use crate::config::Config;
use crate::errors::Error;
use crate::prelude::Result;
use crate::{CLIENT_MAP, CONFIG, PACKET_BUFFER, SERVER_IS_RUNNING, WORKER_CONDVAR};

pub async fn init_job_system() -> Result<()> {
    let num_cores = num_cpus::get(); // Get the number of physical cores

    let mut workers: Vec<JoinHandle<()>> = Vec::with_capacity(num_cores);

    // Create a worker for each core except one for the main thread and one for the life signal checker
    for _ in 0..num_cores - 2 {
        workers.push(task::spawn_blocking(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(js_worker());
        }));
    }
    workers.push(task::spawn_blocking(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(js_life_signal_checker());
    }));

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
                // If no packet : lock the thread until there is some packet to compute. Sort of a sleep mode
                let (ref lock, ref cvar) = &*WORKER_CONDVAR.clone();
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

async fn js_life_signal_checker() {
    'server_life: while SERVER_IS_RUNNING.load(Ordering::Acquire) {
        trace!("[HeartBeat-Checker] Checking life signals for each connected clients...");
        // 1 - loop over all clients
        'iter: for client_info in CLIENT_MAP.iter() {
            // 2 - check if the last ping is older than HEARTBEAT_PERIOD ms
            trace!("[HeartBeat-Checker] Checking life signal for client {}", client_info.key());
            let client_read = client_info.value().read().await;
            let last_life_signe = {
                client_read.life_signe.read().await.elapsed().as_millis()
            };

            if last_life_signe < CONFIG.heart_beat_period as u128 {
                // 3 - Last ping is close enough, we can continue
                trace!("[HeartBeat-Checker] Client {} is alive.", client_info.key());
                continue 'iter;
            }
            
            if last_life_signe > (CONFIG.heart_beat_period as u128 * 2) {
                // 4 - Last ping is older than 2 * HEARTBEAT_PERIOD ms, disconnect the client
                // TODO : Implement the disconnection process and call it here
                info!("[HeartBeat-Checker] Client {} is dead.", client_info.key());
                continue 'iter;
            }
            
            
            // 5 - Last ping is older than HEARTBEAT_PERIOD ms, send a ping
            let mut status = client_read.status.write().await;
            match *status {
                Connecting | Connected | Unknown => {
                    // Set the connection as spurious
                    *status = Spurious;
                    // Send a heartbeat_request
                    let sender = match &client_read.sender {
                        Some(sender) => sender.clone(),
                        None => {
                            error!("[HeartBeat-Checker] Client {} has no sender stream!", client_info.key());
                            // Robust implementation MUST disconnect the client here
                            continue 'iter;
                        }
                    };
            
                    let dtg = DtgHeartbeatRequest::new();
                    sender.write().await.write_all(&dtg.as_bytes()).await;
                }
                Spurious => {
                    // TODO : Implement the disconnection process and call it here
                }
            }
        }
        // sleep for the quarter of the heartbeat period
        tokio::time::sleep(tokio::time::Duration::from_millis((CONFIG.heart_beat_period / 4) as u64)).await;
    }
}
