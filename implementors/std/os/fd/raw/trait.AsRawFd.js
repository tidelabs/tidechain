(function() {var implementors = {};
implementors["async_io"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"async_io/struct.Async.html\" title=\"struct async_io::Async\">Async</a>&lt;T&gt;","synthetic":false,"types":["async_io::Async"]}];
implementors["futures_rustls"] = [{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"futures_rustls/client/struct.TlsStream.html\" title=\"struct futures_rustls::client::TlsStream\">TlsStream</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>,&nbsp;</span>","synthetic":false,"types":["futures_rustls::client::TlsStream"]},{"text":"impl&lt;IO&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"futures_rustls/server/struct.TlsStream.html\" title=\"struct futures_rustls::server::TlsStream\">TlsStream</a>&lt;IO&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;IO: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>,&nbsp;</span>","synthetic":false,"types":["futures_rustls::server::TlsStream"]},{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"enum\" href=\"futures_rustls/enum.TlsStream.html\" title=\"enum futures_rustls::TlsStream\">TlsStream</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>,&nbsp;</span>","synthetic":false,"types":["futures_rustls::TlsStream"]}];
implementors["hyper"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"hyper/server/conn/struct.AddrStream.html\" title=\"struct hyper::server::conn::AddrStream\">AddrStream</a>","synthetic":false,"types":["hyper::server::tcp::addr_stream::AddrStream"]}];
implementors["io_lifetimes"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"io_lifetimes/struct.BorrowedFd.html\" title=\"struct io_lifetimes::BorrowedFd\">BorrowedFd</a>&lt;'_&gt;","synthetic":false,"types":["io_lifetimes::types::BorrowedFd"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"io_lifetimes/struct.OwnedFd.html\" title=\"struct io_lifetimes::OwnedFd\">OwnedFd</a>","synthetic":false,"types":["io_lifetimes::types::OwnedFd"]}];
implementors["memfd"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"memfd/struct.Memfd.html\" title=\"struct memfd::Memfd\">Memfd</a>","synthetic":false,"types":["memfd::memfd::Memfd"]}];
implementors["mio"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/struct.Poll.html\" title=\"struct mio::Poll\">Poll</a>","synthetic":false,"types":["mio::poll::Poll"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/struct.Registry.html\" title=\"struct mio::Registry\">Registry</a>","synthetic":false,"types":["mio::poll::Registry"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/unix/pipe/struct.Sender.html\" title=\"struct mio::unix::pipe::Sender\">Sender</a>","synthetic":false,"types":["mio::sys::unix::pipe::Sender"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/unix/pipe/struct.Receiver.html\" title=\"struct mio::unix::pipe::Receiver\">Receiver</a>","synthetic":false,"types":["mio::sys::unix::pipe::Receiver"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/net/struct.TcpListener.html\" title=\"struct mio::net::TcpListener\">TcpListener</a>","synthetic":false,"types":["mio::net::tcp::listener::TcpListener"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/net/struct.TcpStream.html\" title=\"struct mio::net::TcpStream\">TcpStream</a>","synthetic":false,"types":["mio::net::tcp::stream::TcpStream"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/net/struct.UdpSocket.html\" title=\"struct mio::net::UdpSocket\">UdpSocket</a>","synthetic":false,"types":["mio::net::udp::UdpSocket"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/net/struct.UnixDatagram.html\" title=\"struct mio::net::UnixDatagram\">UnixDatagram</a>","synthetic":false,"types":["mio::net::uds::datagram::UnixDatagram"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/net/struct.UnixListener.html\" title=\"struct mio::net::UnixListener\">UnixListener</a>","synthetic":false,"types":["mio::net::uds::listener::UnixListener"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"mio/net/struct.UnixStream.html\" title=\"struct mio::net::UnixStream\">UnixStream</a>","synthetic":false,"types":["mio::net::uds::stream::UnixStream"]}];
implementors["netlink_sys"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"netlink_sys/struct.Socket.html\" title=\"struct netlink_sys::Socket\">Socket</a>","synthetic":false,"types":["netlink_sys::socket::Socket"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"netlink_sys/struct.SmolSocket.html\" title=\"struct netlink_sys::SmolSocket\">SmolSocket</a>","synthetic":false,"types":["netlink_sys::smol::SmolSocket"]}];
implementors["nix"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"nix/dir/struct.Dir.html\" title=\"struct nix::dir::Dir\">Dir</a>","synthetic":false,"types":["nix::dir::Dir"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"nix/pty/struct.PtyMaster.html\" title=\"struct nix::pty::PtyMaster\">PtyMaster</a>","synthetic":false,"types":["nix::pty::PtyMaster"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"nix/sys/signalfd/struct.SignalFd.html\" title=\"struct nix::sys::signalfd::SignalFd\">SignalFd</a>","synthetic":false,"types":["nix::sys::signalfd::SignalFd"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"nix/sys/inotify/struct.Inotify.html\" title=\"struct nix::sys::inotify::Inotify\">Inotify</a>","synthetic":false,"types":["nix::sys::inotify::Inotify"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"nix/sys/timerfd/struct.TimerFd.html\" title=\"struct nix::sys::timerfd::TimerFd\">TimerFd</a>","synthetic":false,"types":["nix::sys::timerfd::TimerFd"]}];
implementors["socket2"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"socket2/struct.Socket.html\" title=\"struct socket2::Socket\">Socket</a>","synthetic":false,"types":["socket2::socket::Socket"]}];
implementors["tempfile"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tempfile/struct.NamedTempFile.html\" title=\"struct tempfile::NamedTempFile\">NamedTempFile</a>","synthetic":false,"types":["tempfile::file::NamedTempFile"]}];
implementors["tokio"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/io/unix/struct.AsyncFd.html\" title=\"struct tokio::io::unix::AsyncFd\">AsyncFd</a>&lt;T&gt;","synthetic":false,"types":["tokio::io::async_fd::AsyncFd"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/net/struct.TcpListener.html\" title=\"struct tokio::net::TcpListener\">TcpListener</a>","synthetic":false,"types":["tokio::net::tcp::listener::TcpListener"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/net/struct.TcpSocket.html\" title=\"struct tokio::net::TcpSocket\">TcpSocket</a>","synthetic":false,"types":["tokio::net::tcp::socket::TcpSocket"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/net/struct.TcpStream.html\" title=\"struct tokio::net::TcpStream\">TcpStream</a>","synthetic":false,"types":["tokio::net::tcp::stream::TcpStream"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/net/struct.UdpSocket.html\" title=\"struct tokio::net::UdpSocket\">UdpSocket</a>","synthetic":false,"types":["tokio::net::udp::UdpSocket"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/net/struct.UnixDatagram.html\" title=\"struct tokio::net::UnixDatagram\">UnixDatagram</a>","synthetic":false,"types":["tokio::net::unix::datagram::socket::UnixDatagram"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/net/struct.UnixListener.html\" title=\"struct tokio::net::UnixListener\">UnixListener</a>","synthetic":false,"types":["tokio::net::unix::listener::UnixListener"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio/net/struct.UnixStream.html\" title=\"struct tokio::net::UnixStream\">UnixStream</a>","synthetic":false,"types":["tokio::net::unix::stream::UnixStream"]}];
implementors["tokio_rustls"] = [{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio_rustls/client/struct.TlsStream.html\" title=\"struct tokio_rustls::client::TlsStream\">TlsStream</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>,&nbsp;</span>","synthetic":false,"types":["tokio_rustls::client::TlsStream"]},{"text":"impl&lt;IO&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio_rustls/server/struct.TlsStream.html\" title=\"struct tokio_rustls::server::TlsStream\">TlsStream</a>&lt;IO&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;IO: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>,&nbsp;</span>","synthetic":false,"types":["tokio_rustls::server::TlsStream"]},{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"enum\" href=\"tokio_rustls/enum.TlsStream.html\" title=\"enum tokio_rustls::TlsStream\">TlsStream</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>,&nbsp;</span>","synthetic":false,"types":["tokio_rustls::TlsStream"]}];
implementors["tokio_util"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a> for <a class=\"struct\" href=\"tokio_util/compat/struct.Compat.html\" title=\"struct tokio_util::compat::Compat\">Compat</a>&lt;T&gt;","synthetic":false,"types":["tokio_util::compat::Compat"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()