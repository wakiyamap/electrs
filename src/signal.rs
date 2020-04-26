use std::time::Duration;

use crate::errors::*;

#[derive(Clone)] // so multiple threads could wait on signals
pub struct Waiter {
    signal: chan::Receiver<chan_signal::Signal>,
}

impl Waiter {
    pub fn start() -> Waiter {
        Waiter {
            signal: chan_signal::notify(&[chan_signal::Signal::INT, chan_signal::Signal::TERM]),
        }
    }
    pub fn wait(&self, duration: Duration) -> Result<()> {
        let signal = &self.signal;
        let timeout = chan::after(duration);
        chan_select! {
            signal.recv() -> s => {
                if let Some(sig) = s {
                    bail!(ErrorKind::Interrupt(sig));
                }
            },
            timeout.recv() => {},
        }
        Ok(())
    }
    pub fn poll(&self) -> Result<()> {
        self.wait(Duration::from_secs(0))
    }
}
