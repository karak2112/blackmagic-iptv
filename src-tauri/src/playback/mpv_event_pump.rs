use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use libmpv2::events::{Event, EventContext};
use libmpv2::Mpv;

/// Drains libmpv's event queue on a dedicated thread.
///
/// Without this, synchronous mpv commands can block forever once the internal
/// event ring buffer fills up (common on Windows with embedded `wid` playback).
pub struct MpvEventPump {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

impl MpvEventPump {
    pub fn spawn(mpv: &Mpv) -> Self {
        let ctx = mpv.ctx;
        let stop = Arc::new(AtomicBool::new(false));
        let (wake_tx, wake_rx) = mpsc::channel();
        let stop_flag = stop.clone();

        let mut ev_ctx = EventContext::new(ctx);
        ev_ctx.set_wakeup_callback(move || {
            let _ = wake_tx.send(());
        });

        let thread = thread::Builder::new()
            .name("mpv-events".into())
            .spawn(move || {
                while !stop_flag.load(Ordering::Relaxed) {
                    let _ = wake_rx.recv_timeout(Duration::from_millis(100));
                    drain_events(&mut ev_ctx);
                }
                drain_events(&mut ev_ctx);
            })
            .expect("mpv event pump thread");

        Self { stop, thread }
    }

    pub fn stop(self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.thread.join();
    }
}

fn drain_events(ev_ctx: &mut EventContext) {
    loop {
        match ev_ctx.wait_event(0.0) {
            None => break,
            Some(Ok(Event::QueueOverflow)) => tracing::warn!("mpv event queue overflow"),
            Some(Ok(_)) => {}
            Some(Err(e)) => tracing::debug!("mpv event error: {e}"),
        }
    }
}
