/// Used to send a result to a [`Promise`].
///
/// You must call [`Self::send`] with a value eventually.
///
/// If you drop the `Sender` without putting a value into it,
/// it will cause the connected [`Promise`] to panic when polled.
#[must_use = "You should call Sender::send with the result"]
pub struct Sender<T>(std::sync::mpsc::Sender<T>);

impl<T> Sender<T> {
    /// Send the result to the [`Promise`].
    ///
    /// If the [`Promise`] has dropped, this does nothing.
    pub fn send(self, value: T) {
        self.0.send(value).ok(); // We ignore the error caused by the receiver being dropped.
    }
}

// ----------------------------------------------------------------------------

/// A promise that waits for the reception of a single value,
/// presumably from some async task.
///
/// A `Promise` starts out waiting for a value.
/// Each time you call a member method it will check if that value is ready.
/// Once ready, the `Promise` will store the value until you drop the `Promise`.
///
/// Example:
///
/// ```
/// # fn something_slow() {}
/// # use poll_promise::Promise;
/// #
/// let promise = Promise::spawn_thread("slow_operation", move || something_slow());
///
/// // Then in the game loop or immediate mode GUI code:
/// if let Some(result) = promise.ready() {
///     // Use/show result
/// } else {
///     // Show a loading screen
/// }
/// ```
///
/// If you enable the `tokio` feature you can use `poll-promise` with the [tokio](https://github.com/tokio-rs/tokio)
/// runtime to run `async` tasks using [`Promise::spawn_async`] and [`Promise::spawn_blocking`].
#[must_use]
pub struct Promise<T: Send + 'static> {
    data: PromiseImpl<T>,
}

// Ensure that Promise is !Sync, confirming the safety of the unsafe code.
static_assertions::assert_not_impl_all!(Promise<u32>: Sync);
static_assertions::assert_impl_all!(Promise<u32>: Send);

impl<T: Send + 'static> Promise<T> {
    /// Create a [`Promise`] and a corresponding [`Sender`].
    ///
    /// Put the promised value into the sender when it is ready.
    /// If you drop the `Sender` without putting a value into it,
    /// it will cause a panic when polling the `Promise`.
    ///
    /// See also [`Self::spawn_blocking`], [`Self::spawn_async`] and [`Self::spawn_thread`].
    pub fn new() -> (Sender<T>, Self) {
        // We need a channel that we can wait blocking on (for `Self::block_until_ready`).
        // (`tokio::sync::oneshot` does not support blocking receive).
        let (tx, rx) = std::sync::mpsc::channel();
        (
            Sender(tx),
            Self {
                data: PromiseImpl::Pending(rx),
            },
        )
    }

    /// Create a promise that already has the result.
    pub fn from_ready(value: T) -> Self {
        Self {
            data: PromiseImpl::Ready(value),
        }
    }

    /// Spawn a blocking closure in a background thread.
    ///
    /// The first argument is the name of the thread you spawn, passed to [`std::thread::Builder::name`].
    /// It shows up in panic messages.
    ///
    /// This is a convenience method, using [`Self::new`] and [`std::thread::Builder`].
    ///
    /// If you are compiling with the "tokio" feature, you may want to use [`Self::spawn_blocking`] or [`Self::spawn_async`] instead.
    ///
    /// ```
    /// # fn something_slow() {}
    /// # use poll_promise::Promise;
    /// let promise = Promise::spawn_thread("slow_operation", move || something_slow());
    /// ```
    pub fn spawn_thread<F>(thread_name: impl Into<String>, f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (sender, promise) = Self::new();
        std::thread::Builder::new()
            .name(thread_name.into())
            .spawn(move || sender.send(f()))
            .expect("Failed to spawn thread");
        promise
    }

    /// Spawn a blocking closure in a tokio task.
    ///
    /// This function is only available if you compile `poll-promise` with the "tokio" feature.
    ///
    /// This is a simple mechanism to offload a heavy function/closure to be processed in the thread pool for blocking CPU work.
    ///
    /// It can't do any async code. For that, use [`Self::spawn_async`].
    ///
    /// This is a convenience method, using [`Self::new`] with [`tokio::task::spawn`] and [`tokio::task::block_in_place`].
    ///
    /// ``` no_run
    /// # fn something_cpu_intensive() {}
    /// # use poll_promise::Promise;
    /// let promise = Promise::spawn_blocking(move || something_cpu_intensive());
    /// ```
    #[cfg(feature = "tokio")]
    pub fn spawn_blocking<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (sender, promise) = Self::new();
        tokio::task::spawn(async move { sender.send(tokio::task::block_in_place(f)) });
        promise
    }

    /// Spawn a light-weight future.
    ///
    /// This function is only available if you compile `poll-promise` with the "tokio" feature.
    ///
    /// This should be used for spawning asynchronous work that does _not_ do any heavy CPU computations
    /// as that will block other spawned tasks and will delay them. For example network IO, timers, etc.
    ///
    /// These type of future can have manually blocking code within it though, but has to then manually use
    /// [`tokio::task::block_in_place`](https://docs.rs/tokio/1.15.0/tokio/task/fn.block_in_place.html) on that,
    /// or `.await` that future.
    ///
    /// If you have a function or closure that you just want to offload to processed in the background, use the [`Self::spawn_blocking`] function instead.
    ///
    /// See the [tokio docs](https://docs.rs/tokio/1.15.0/tokio/index.html#cpu-bound-tasks-and-blocking-code) for more details about
    /// CPU-bound tasks vs async IO tasks.
    ///
    /// This is a convenience method, using [`Self::new`] with [`tokio::task::spawn`].
    ///
    /// ``` no_run
    /// # async fn something_async() {}
    /// # use poll_promise::Promise;
    /// let promise = Promise::spawn_async(async move { something_async().await });
    /// ```
    #[cfg(feature = "tokio")]
    pub fn spawn_async(future: impl std::future::Future<Output = T> + 'static + Send) -> Self {
        let (sender, promise) = Self::new();
        tokio::task::spawn(async move { sender.send(future.await) });
        promise
    }

    /// Polls the promise and either returns a reference to the data, or [`None`] if still pending.
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn ready(&self) -> Option<&T> {
        match self.poll() {
            std::task::Poll::Pending => None,
            std::task::Poll::Ready(value) => Some(value),
        }
    }

    /// Polls the promise and either returns a mutable reference to the data, or [`None`] if still pending.
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn ready_mut(&mut self) -> Option<&mut T> {
        match self.poll_mut() {
            std::task::Poll::Pending => None,
            std::task::Poll::Ready(value) => Some(value),
        }
    }

    /// Returns either the completed promise object or the promise itself if it is not completed yet.
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn try_take(self) -> Result<T, Self> {
        self.data.try_take().map_err(|data| Self { data })
    }

    /// Block execution until ready, then returns a reference to the value.
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn block_until_ready(&self) -> &T {
        self.data.block_until_ready()
    }

    /// Block execution until ready, then returns a mutable reference to the value.
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn block_until_ready_mut(&mut self) -> &mut T {
        self.data.block_until_ready_mut()
    }

    /// Block execution until ready, then returns the promised value and consumes the `Promise`.
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn block_and_take(self) -> T {
        self.data.block_until_ready();
        match self.data {
            PromiseImpl::Pending(_) => unreachable!(),
            PromiseImpl::Ready(value) => value,
        }
    }

    /// Returns either a reference to the ready value [`std::task::Poll::Ready`]
    /// or [`std::task::Poll::Pending`].
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn poll(&self) -> std::task::Poll<&T> {
        self.data.poll()
    }

    /// Returns either a mut reference to the ready value in a [`std::task::Poll::Ready`]
    /// or a [`std::task::Poll::Pending`].
    ///
    /// Panics if the connected [`Sender`] was dropped before a value was sent.
    pub fn poll_mut(&mut self) -> std::task::Poll<&mut T> {
        self.data.poll_mut()
    }
}

// ----------------------------------------------------------------------------

enum PromiseImpl<T: Send + 'static> {
    Pending(std::sync::mpsc::Receiver<T>),
    Ready(T),
}

impl<T: Send + 'static> PromiseImpl<T> {
    fn poll_mut(&mut self) -> std::task::Poll<&mut T> {
        match self {
            Self::Pending(rx) => {
                if let Ok(value) = rx.try_recv() {
                    *self = Self::Ready(value);
                    match self {
                        Self::Ready(ref mut value) => std::task::Poll::Ready(value),
                        Self::Pending(_) => unreachable!(),
                    }
                } else {
                    std::task::Poll::Pending
                }
            }
            Self::Ready(ref mut value) => std::task::Poll::Ready(value),
        }
    }

    /// Returns either the completed promise object or the promise itself if it is not completed yet.
    fn try_take(self) -> Result<T, Self> {
        match self {
            Self::Pending(ref rx) => match rx.try_recv() {
                Ok(value) => Ok(value),
                Err(std::sync::mpsc::TryRecvError::Empty) => Err(self),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    panic!("The Promise Sender was dropped")
                }
            },
            Self::Ready(value) => Ok(value),
        }
    }

    #[allow(unsafe_code)]
    fn poll(&self) -> std::task::Poll<&T> {
        match self {
            Self::Pending(rx) => {
                match rx.try_recv() {
                    Ok(value) => {
                        // SAFETY: This is safe since Promise (and PromiseData) are !Sync and thus
                        // need external synchronization anyway. We can only transition from
                        // Pending->Ready, not the other way around, so once we're Ready we'll
                        // stay ready.
                        unsafe {
                            let myself = self as *const Self as *mut Self;
                            *myself = Self::Ready(value);
                        }
                        match self {
                            Self::Ready(ref value) => std::task::Poll::Ready(value),
                            Self::Pending(_) => unreachable!(),
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => std::task::Poll::Pending,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        panic!("The Promise Sender was dropped")
                    }
                }
            }
            Self::Ready(ref value) => std::task::Poll::Ready(value),
        }
    }

    fn block_until_ready_mut(&mut self) -> &mut T {
        match self {
            Self::Pending(rx) => {
                let value = rx.recv().expect("The Promise Sender was dropped");
                *self = Self::Ready(value);
                match self {
                    Self::Ready(ref mut value) => value,
                    Self::Pending(_) => unreachable!(),
                }
            }
            Self::Ready(ref mut value) => value,
        }
    }

    #[allow(unsafe_code)]
    fn block_until_ready(&self) -> &T {
        match self {
            Self::Pending(rx) => {
                let value = rx.recv().expect("The Promise Sender was dropped");
                // SAFETY: This is safe since `Promise` (and `PromiseData`) are `!Sync` and thus
                // need external synchronization anyway. We can only transition from
                // Pending->Ready, not the other way around, so once we're Ready we'll
                // stay ready.
                unsafe {
                    let myself = self as *const Self as *mut Self;
                    *myself = Self::Ready(value);
                }
                match self {
                    Self::Ready(ref value) => value,
                    Self::Pending(_) => unreachable!(),
                }
            }
            Self::Ready(ref value) => value,
        }
    }
}
