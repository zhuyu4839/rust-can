#[cfg(not(feature = "async"))]
pub use self::sync::IsoTpAdapter;
#[cfg(feature = "async")]
pub use self::r#async::IsoTpAdapter;

// use rs_can::{Listener, CanDriver, Frame};
//
// impl<D, C, F> IsoTpAdapter<D, C, F>
// where
//     D: CanDriver<Channel = C, Frame = F> + Clone + 'static,
//     C: Clone + Display + 'static,
//     F: Frame<Channel = C> + Clone + Send + Display + 'static,
//
// }

#[cfg(not(feature = "async"))]
pub mod sync {
    use std::{collections::HashMap, fmt::Display, thread, sync::{Arc, Mutex, MutexGuard, Weak, mpsc::{channel, Sender, Receiver}}};
    use std::time::Duration;
    use crate::{Listener, CanDriver, Frame};

    #[derive(Clone)]
    pub struct IsoTpAdapter<D, C, F> {
        pub(crate) device: D,
        pub(crate) sender: Sender<F>,
        pub(crate) receiver: Arc<Mutex<Receiver<F>>>,
        pub(crate) listeners: Arc<Mutex<HashMap<String, Box<dyn Listener<C, F>>>>>,
        pub(crate) stop_tx: Sender<()>,
        pub(crate) stop_rx: Arc<Mutex<Receiver<()>>>,
        pub(crate) send_task: Weak<thread::JoinHandle<()>>,
        pub(crate) receive_task: Weak<thread::JoinHandle<()>>,
        pub(crate) interval: Option<u64>,
    }

    impl<D, C, F> IsoTpAdapter<D, C, F>
    where
        D: CanDriver<Channel = C, Frame = F> + Clone + Send + 'static,
        C: Clone + Display + 'static,
        F: Frame<Channel = C> + Clone + Send + Display + 'static,
    {
        pub fn new(device: D) -> Self {
            let (tx, rx) = channel();
            let (stop_tx, stop_rx) = channel();
            Self {
                device,
                sender: tx,
                receiver: Arc::new(Mutex::new(rx)),
                listeners: Arc::new(Mutex::new(HashMap::new())),
                stop_tx,
                stop_rx: Arc::new(Mutex::new(stop_rx)),
                send_task: Default::default(),
                receive_task: Default::default(),
                interval: Default::default(),
            }
        }

        #[inline]
        pub fn register_listener(&self, name: String, listener: Box<dyn Listener<C, F>>) -> bool {
            log::trace!("SyncISO-TP - register listener {}", name);
            match self.listeners.lock() {
                Ok(mut listeners) => {
                    listeners.insert(name, listener);
                    true
                },
                Err(e) => {
                    log::warn!("SyncISO-TP - listener error {} when registering listener {}", e, name);
                    false
                },
            }
        }

        pub fn unregister_listener(&self, name: &str) -> bool {
            log::trace!("SyncISO-TP - unregister listener {}", name);
            match self.listeners.lock() {
                Ok(mut listeners) => {
                    listeners.remove(name);
                    true
                }
                Err(e) => {
                    log::warn!("SyncISO-TP - listener error {} when unregistering listener {}", e, name);
                    false
                }
            }
        }

        pub fn unregister_all_listeners(&self) -> bool {
            match self.listeners.lock() {
                Ok(mut listeners) => {
                    listeners.clear();
                    true
                },
                Err(e) => {
                    log::warn!("SyncISO-TP - listener error {} when unregistering all listeners", e);
                    false
                }
            }
        }

        pub fn listener_names(&self) -> Vec<String> {
            match self.listeners.lock() {
                Ok(v) => {
                    v.keys()
                        .into_iter()
                        .map(|f| f.clone())
                        .collect()
                },
                Err(e) => {
                    log::warn!("SyncISO-TP - listener error {} when get all listener names", e);
                    vec![]
                },
            }
        }

        pub fn listener_callback(&self, name: &str, callback: impl FnOnce(&Box<dyn Listener<C, F>>)) {
            match self.listeners.lock() {
                Ok(listeners) => {
                    if let Some(listener) = listeners.get(name) {
                        callback(listener);
                    }
                },
                Err(e) => {
                    log::warn!("SyncISO-TP - listener error {} when trying to callback", e);
                }
            }
        }

        #[inline]
        pub fn sender(&self) -> Sender<F> {
            self.sender.clone()
        }

        pub fn start(&mut self, interval_us: u64) {
            self.interval = Some(interval_us);

            let self_arc = Arc::new(Mutex::new(self.clone()));
            let stop_rx = Arc::clone(&self.stop_rx);
            let tx_task = thread::spawn(move || {
                if let Ok(self_clone) = self_arc.lock() {
                    Self::transmit_loop(self_clone, interval_us, Arc::clone(&stop_rx));
                }
            });

            let self_arc = Arc::new(Mutex::new(self.clone()));
            let stop_rx = Arc::clone(&self.stop_rx);
            let rx_task = thread::spawn(move || {
                if let Ok(self_clone) = self_arc.lock() {
                    Self::receive_loop(self_clone, interval_us, Arc::clone(&stop_rx));
                }
            });

            self.send_task = Arc::downgrade(&Arc::new(tx_task));
            self.receive_task = Arc::downgrade(&Arc::new(rx_task));
        }

        pub fn stop(&mut self) {
            log::info!("SyncISO-TP - stopping adapter");
            if let Err(e) = self.stop_tx.send(()) {
                log::warn!("SyncISO-TP - error {} when stopping transmit", e);
            }

            thread::sleep(Duration::from_micros(2 * self.interval.unwrap_or(50 * 1000)));

            if let Some(task) = self.send_task.upgrade() {
                if !task.is_finished() {
                    log::warn!("SyncISO-TP - transmit task is running after stop signal");
                }
            }

            if let Some(task) = self.receive_task.upgrade() {
                if !task.is_finished() {
                    log::warn!("SyncISO-TP - receive task is running after stop signal");
                }
            }

            self.device.shutdown();
        }

        fn transmit_loop(device: MutexGuard<Self>, interval_us: u64, stopper: Arc<Mutex<Receiver<()>>>) {
            Self::loop_util(
                device,
                interval_us,
                stopper,
                |d| Self::transmit_callback(&d.receiver, &d.device, &d.listeners, None)
            );
        }

        fn receive_loop(device: MutexGuard<Self>, interval_us: u64, stopper: Arc<Mutex<Receiver<()>>>) {
            Self::loop_util(
                device,
                interval_us,
                stopper,
                |d| Self::receive_callback(&d.device, &d.listeners, None)
            )
        }

        fn transmit_callback(receiver: &Arc<Mutex<Receiver<F>>>, device: &D, listeners: &Arc<Mutex<HashMap<String, Box<dyn Listener<C, F>>>>>, timeout: Option<u32>) {
            if let Ok(receiver) = receiver.lock() {
                if let Ok(msg) = receiver.try_recv() {
                    log::trace!("SyncISO-TP - transmitting: {}", msg);
                    let id = msg.id();
                    let chl = msg.channel();
                    match listeners.lock() {
                        Ok(listeners) => {
                            listeners.values()
                                .for_each(|l| l.on_frame_transmitting(chl.clone(), &msg));
                        },
                        Err(e) => {
                            log::warn!("SyncISO-TP - listener error {} when notify transmitting listeners", e);
                        }
                    }

                    match device.transmit(msg, timeout) {
                        Ok(_) => match listeners.lock() {
                            Ok(listeners) => {
                                listeners.values()
                                    .for_each(|l| l.on_frame_transmitted(chl.clone(), id));
                            },
                            Err(e) => {
                                log::warn!("SyncISO-TP - listener error {:?} when notify transmitted listeners", e);
                            }
                        },
                        Err(e) => {
                            log::warn!("SyncISO-TP - error {} when transmitting message", e);
                        }
                    }
                }
            }
        }

        fn receive_callback(device: &D, listeners: &Arc<Mutex<HashMap<String, Box<dyn Listener<C, F>>>>>, timeout: Option<u32>) {
            let channels = device.opened_channels();
            channels.into_iter()
                .for_each(|c| {
                    if let Ok(messages) = device.receive(c.clone(), timeout) {
                        if !messages.is_empty() {
                            match listeners.lock() {
                                Ok(listeners) => {
                                    listeners.values()
                                        .for_each(|l| l.on_frame_received(c.clone(), &messages));
                                }
                                Err(e) => {
                                    log::warn!("SyncISO-TP - listener error {:?} when notify received listeners", e);
                                }
                            }
                        }
                    }
                });
        }

        fn loop_util(device: MutexGuard<Self>, interval: u64, stopper: Arc<Mutex<Receiver<()>>>, callback: fn(&MutexGuard<Self>)) {
            loop {
                if device.device.is_closed() {
                    log::info!("SyncISO-TP - device closed");
                    break;
                }

                callback(&device);

                if let Ok(stopper) = stopper.try_lock() {
                    if let Ok(()) = stopper.try_recv() {
                        log::info!("SyncISO-TP - stop sync");
                        break;
                    }
                }

                thread::sleep(Duration::from_micros(interval));
            }
        }
    }
}

#[cfg(feature = "async")]
pub(crate) mod r#async {
    pub(crate) struct IsoTpAdapter<D, C, F> {
        pub(crate) device: D,
        pub(crate) sender: Sender<F>,
        pub(crate) receiver: Arc<Mutex<Receiver<F>>>,
        pub(crate) listeners: Arc<Mutex<HashMap<String, Box<dyn Listener<C, F>>>>>,
        pub(crate) stop_tx: Sender<()>,
        pub(crate) stop_rx: Arc<Mutex<Receiver<()>>>,
        pub(crate) send_task: Weak<thread::JoinHandle<()>>,
        pub(crate) receive_task: Weak<thread::JoinHandle<()>>,
        pub(crate) interval: Option<u64>,
    }
}
