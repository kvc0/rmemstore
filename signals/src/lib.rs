use std::os::raw::c_int;

pub struct Signals {
    signal_queue: tokio::sync::mpsc::Receiver<Signal>,
}

#[derive(Clone, Copy)]
pub enum Signal {
    Sigint,
    Sigterm,
    Sighup,
    Sigabrt,
    Sigquit,
}

impl Signal {
    fn as_signal_int(&self) -> c_int {
        match self {
            Signal::Sigint => signal_hook::consts::SIGINT,
            Signal::Sigterm => signal_hook::consts::SIGTERM,
            Signal::Sighup => signal_hook::consts::SIGHUP,
            Signal::Sigabrt => signal_hook::consts::SIGABRT,
            Signal::Sigquit => signal_hook::consts::SIGQUIT,
            // Signal::Sigkill => signal_hook::consts::SIGKILL, // not allowed
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Signal::Sigint => "sigint",
            Signal::Sigterm => "sigterm",
            Signal::Sighup => "sighup",
            Signal::Sigabrt => "sigabrt",
            Signal::Sigquit => "sigquit",
        }
    }
}

impl Signals {
    pub fn register() -> std::io::Result<Self> {
        let (signal_sender, signal_queue) = tokio::sync::mpsc::channel(16);
        let signals = Self { signal_queue };

        Self::register_signal(Signal::Sigint, signal_sender.clone())?;
        Self::register_signal(Signal::Sigterm, signal_sender.clone())?;
        Self::register_signal(Signal::Sighup, signal_sender.clone())?;

        Ok(signals)
    }

    pub async fn wait_for_termination(self) {
        let mut signals = self.signal_queue;
        loop {
            match signals.recv().await {
                Some(signal) => match signal {
                    Signal::Sigint => {
                        log::info!("resolving future for {}", signal.name());
                        break;
                    }
                    Signal::Sigterm => {
                        log::warn!("resolving future for {}", signal.name());
                        break;
                    }
                    Signal::Sigabrt => {
                        log::info!("resolving future for {}", signal.name());
                    }
                    Signal::Sigquit => {
                        log::info!("resolving future for {}", signal.name());
                    }
                    Signal::Sighup => {
                        log::info!("nothing to do for {}", signal.name());
                    }
                },
                None => {
                    log::trace!("no signals to handle.");
                }
            }
        }
    }

    fn register_signal(
        signal: Signal,
        queue: tokio::sync::mpsc::Sender<Signal>,
    ) -> std::io::Result<()> {
        unsafe {
            signal_hook::low_level::register(signal.as_signal_int(), move || {
                log::warn!("Received {}", signal.name());
                match queue.try_send(signal) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Dropped {}: {e:?}", signal.name());
                    }
                }
            })
        }?;
        Ok(())
    }
}
