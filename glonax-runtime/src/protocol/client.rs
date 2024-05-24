use std::time::Duration;

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::protocol::{frame, Stream};

/// A builder for creating a Glonax client.
///
/// The `ClientBuilder` struct provides a fluent interface for configuring and creating a Glonax client.
/// It allows setting options such as the address, session name, control mode, command mode, failsafe mode, and stream mode.
/// Once the desired options are set, the `connect` method can be called to establish a connection and return a `Stream` and `Instance`.
///
/// # Example
///
/// ```no_run
/// use glonax::protocol::client::{ClientBuilder, connect};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let address = "127.0.0.1:8080";
///     let session_name = "my_session";
///
///     let (stream, instance) = ClientBuilder::new(address, session_name)
///         .control(true)
///         .command(true)
///         .failsafe(false)
///         .stream(false)
///         .connect()
///         .await?;
///
///     // Use the `stream` and `instance` here...
///
///     Ok(())
/// }
/// ```
pub struct ClientBuilder<A: ToSocketAddrs> {
    address: A,
    session_name: String,
    control: bool,
    command: bool,
    failsafe: bool,
    stream: bool,
}

impl<A: ToSocketAddrs> ClientBuilder<A> {
    /// Creates a new `ClientBuilder` with the specified address and session name.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to connect to.
    /// * `session_name` - The name of the session.
    ///
    /// # Example
    ///
    /// ```rust
    /// use glonax::protocol::client::ClientBuilder;
    ///
    /// let address = "127.0.0.1:8080";
    /// let session_name = "my_session";
    ///
    /// let builder = ClientBuilder::new(address, session_name);
    /// ```
    pub fn new(address: A, session_name: impl ToString) -> Self {
        Self {
            address,
            session_name: session_name.to_string(),
            control: false,
            command: false,
            failsafe: false,
            stream: false,
        }
    }

    /// Sets the control mode for the client.
    ///
    /// # Arguments
    ///
    /// * `control` - A boolean indicating whether control mode should be enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use glonax::protocol::client::ClientBuilder;
    ///
    /// let address = "127.0.0.1:8080";
    /// let session_name = "my_session";
    ///
    /// let builder = ClientBuilder::new(address, session_name)
    ///     .control(true);
    /// ```
    pub fn control(mut self, control: bool) -> Self {
        self.control = control;
        self
    }

    /// Sets the command mode for the client.
    ///
    /// # Arguments
    ///
    /// * `command` - A boolean indicating whether command mode should be enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use glonax::protocol::client::ClientBuilder;
    ///
    /// let address = "127.0.0.1:8080";
    /// let session_name = "my_session";
    ///
    /// let builder = ClientBuilder::new(address, session_name)
    ///     .command(true);
    /// ```
    pub fn command(mut self, command: bool) -> Self {
        self.command = command;
        self
    }

    /// Sets the failsafe mode for the client.
    ///
    /// # Arguments
    ///
    /// * `failsafe` - A boolean indicating whether failsafe mode should be enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use glonax::protocol::client::ClientBuilder;
    ///
    /// let address = "127.0.0.1:8080";
    /// let session_name = "my_session";
    ///
    /// let builder = ClientBuilder::new(address, session_name)
    ///     .failsafe(false);
    /// ```
    pub fn failsafe(mut self, failsafe: bool) -> Self {
        self.failsafe = failsafe;
        self
    }

    /// Sets the stream mode for the client.
    ///
    /// # Arguments
    ///
    /// * `stream` - A boolean indicating whether stream mode should be enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use glonax::protocol::client::ClientBuilder;
    ///
    /// let address = "127.0.0.1:8080";
    /// let session_name = "my_session";
    ///
    /// let builder = ClientBuilder::new(address, session_name)
    ///     .stream(false);
    /// ```
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Establishes a connection to the server and returns a `Stream` and `Instance`.
    ///
    /// # Returns
    ///
    /// A `std::io::Result` containing a tuple of the `Stream` and `Instance` if the connection is successful.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use glonax::protocol::client::{ClientBuilder, connect};
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let address = "127.0.0.1:8080";
    ///     let session_name = "my_session";
    ///
    ///     let (stream, instance) = ClientBuilder::new(address, session_name)
    ///         .connect()
    ///         .await?;
    ///
    ///     // Use the `stream` and `instance` here...
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(self) -> std::io::Result<(Stream<TcpStream>, crate::core::Instance)> {
        let mut flags = 0;

        if self.control {
            flags |= frame::Session::MODE_CONTROL;
        } else {
            flags &= !frame::Session::MODE_CONTROL;
        }

        if self.command {
            flags |= frame::Session::MODE_COMMAND;
        } else {
            flags &= !frame::Session::MODE_COMMAND;
        }

        if self.failsafe {
            flags |= frame::Session::MODE_FAILSAFE;
        } else {
            flags &= !frame::Session::MODE_FAILSAFE;
        }

        if self.stream {
            flags |= frame::Session::MODE_STREAM;
        } else {
            flags &= !frame::Session::MODE_STREAM;
        }

        let stream = tokio::net::TcpStream::connect(self.address).await?;

        let sock_ref = socket2::SockRef::from(&stream);

        let mut keep_alive = socket2::TcpKeepalive::new();
        keep_alive = keep_alive.with_time(Duration::from_secs(2));
        keep_alive = keep_alive.with_interval(Duration::from_secs(2));

        sock_ref.set_tcp_keepalive(&keep_alive)?;
        sock_ref.set_nodelay(true)?;

        let mut client = Stream::new(stream);

        let instance = client.handshake(self.session_name, flags).await?;

        Ok((client, instance))
    }
}

/// Connects to the server using the specified address and session name.
///
/// # Arguments
///
/// * `address` - The address to connect to.
/// * `session_name` - The name of the session.
///
/// # Returns
///
/// A `std::io::Result` containing a tuple of the `Stream` and `Instance` if the connection is successful.
///
/// # Example
///
/// ```no_run
/// use glonax::protocol::client::connect;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let address = "127.0.0.1:8080";
///     let session_name = "my_session";
///
///     let (stream, instance) = connect(address, session_name).await?;
///
///     // Use the `stream` and `instance` here...
///
///     Ok(())
/// }
/// ```
pub async fn connect(
    address: impl ToSocketAddrs,
    session_name: impl ToString,
) -> std::io::Result<(Stream<TcpStream>, crate::core::Instance)> {
    ClientBuilder::new(address, session_name).connect().await
}
