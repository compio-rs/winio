use std::{
    io,
    net::Shutdown,
    os::windows::io::{AsRawSocket, FromRawSocket, IntoRawSocket, RawSocket},
};

use compio_buf::{BufResult, IoBuf, IoBufMut, IoVectoredBuf, IoVectoredBufMut};
use compio_log::debug;
use socket2::{Domain, Protocol, SockAddr, Socket as Socket2, Type};
use windows_sys::Win32::{
    Networking::WinSock::{WSAAsyncSelect, WSARecv, WSASend, FD_ACCEPT, FD_CONNECT},
    UI::WindowsAndMessaging::WM_USER,
};

use crate::{
    syscall_socket, ui::AsRawWindow, wait, winsock_result, with_wsa_overlapped, BufResultExt,
};

const WM_SOCKET: u32 = WM_USER + 1;

#[derive(Debug)]
pub struct Socket {
    socket: Socket2,
}

impl Socket {
    fn from_socket2(socket: Socket2) -> Self {
        Self { socket }
    }

    pub fn peer_addr(&self) -> io::Result<SockAddr> {
        self.socket.peer_addr()
    }

    pub fn local_addr(&self) -> io::Result<SockAddr> {
        self.socket.local_addr()
    }

    pub fn new(domain: Domain, ty: Type, protocol: Option<Protocol>) -> io::Result<Self> {
        let socket = Socket2::new(domain, ty, protocol)?;
        Ok(Self::from_socket2(socket))
    }

    pub fn bind(addr: &SockAddr, ty: Type, protocol: Option<Protocol>) -> io::Result<Self> {
        let socket = Self::new(addr.domain(), ty, protocol)?;
        socket.socket.bind(addr)?;
        Ok(socket)
    }

    pub fn listen(&self, backlog: i32) -> io::Result<()> {
        self.socket.listen(backlog)
    }

    pub async fn connect(&self, addr: &SockAddr, parent: &impl AsRawWindow) -> io::Result<()> {
        let handle = parent.as_raw_window();
        let mut guard = None;
        loop {
            let wait = unsafe { wait(handle, WM_SOCKET) };
            if guard.is_none() {
                guard = Some(WSASelectGuard::new(&parent, &self.socket, FD_CONNECT)?);
            }
            match self.socket.connect(addr) {
                Ok(()) => return Ok(()),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    debug!("connect would block");
                }
                Err(e) => return Err(e),
            }
            let msg = wait.await;
            if msg.wParam == self.socket.as_raw_socket() as _
                && (msg.lParam & 0xFFFFFFFF == FD_CONNECT as _)
            {
                let error_code = msg.lParam >> 32;
                if error_code == 0 {
                    return Ok(());
                } else {
                    return Err(io::Error::from_raw_os_error(error_code as _));
                }
            }
        }
    }

    pub async fn accept(&self, parent: &impl AsRawWindow) -> io::Result<(Self, SockAddr)> {
        let handle = parent.as_raw_window();
        let mut guard = None;
        loop {
            let wait = unsafe { wait(handle, WM_SOCKET) };
            if guard.is_none() {
                guard = Some(WSASelectGuard::new(&parent, &self.socket, FD_ACCEPT)?);
            }
            let msg = wait.await;
            if msg.wParam == self.socket.as_raw_socket() as _
                && (msg.lParam & 0xFFFFFFFF == FD_ACCEPT as _)
            {
                match self.socket.accept() {
                    Ok((socket, addr)) => {
                        // Deregister it on drop and make it blocking.
                        let _ = WSASelectGuard::new(&parent, &socket, 0)?;
                        return Ok((Self::from_socket2(socket), addr));
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(e) => return Err(e),
                }
            }
        }
    }

    pub fn shutdown(&self) -> io::Result<()> {
        self.socket.shutdown(Shutdown::Write)
    }

    pub async fn recv<B: IoBufMut>(&self, buffer: B) -> BufResult<usize, B> {
        debug!("recv");
        with_wsa_overlapped(
            |optr, callback, buffer| unsafe {
                let slice = buffer.as_io_slice_mut();
                let mut flags = 0;
                let mut received = 0;
                let res = WSARecv(
                    self.as_raw_socket() as _,
                    &slice as *const _ as _,
                    1,
                    &mut received,
                    &mut flags,
                    optr,
                    callback,
                );
                winsock_result(res)
            },
            buffer,
        )
        .await
        .map_advanced()
    }

    pub async fn recv_vectored<V: IoVectoredBufMut>(&self, buffer: V) -> BufResult<usize, V> {
        debug!("recv_vectored");
        with_wsa_overlapped(
            |optr, callback, buffer| unsafe {
                let buffers = buffer.as_io_slices_mut();
                let mut flags = 0;
                let mut received = 0;
                let res = WSARecv(
                    self.as_raw_socket() as _,
                    buffers.as_ptr() as _,
                    buffers.len() as _,
                    &mut received,
                    &mut flags,
                    optr,
                    callback,
                );
                winsock_result(res)
            },
            buffer,
        )
        .await
        .map_advanced()
    }

    pub async fn send<T: IoBuf>(&self, buffer: T) -> BufResult<usize, T> {
        debug!("send");
        with_wsa_overlapped(
            |optr, callback, buffer| unsafe {
                let slice = buffer.as_io_slice();
                let mut sent = 0;
                let res = WSASend(
                    self.as_raw_socket() as _,
                    &slice as *const _ as _,
                    1,
                    &mut sent,
                    0,
                    optr,
                    callback,
                );
                winsock_result(res)
            },
            buffer,
        )
        .await
    }

    pub async fn send_vectored<T: IoVectoredBuf>(&self, buffer: T) -> BufResult<usize, T> {
        debug!("send_vectored");
        with_wsa_overlapped(
            |optr, callback, buffer| unsafe {
                let buffers = buffer.as_io_slices();
                let mut sent = 0;
                let res = WSASend(
                    self.as_raw_socket() as _,
                    buffers.as_ptr() as _,
                    buffers.len() as _,
                    &mut sent,
                    0,
                    optr,
                    callback,
                );
                winsock_result(res)
            },
            buffer,
        )
        .await
    }
}

impl AsRawSocket for Socket {
    fn as_raw_socket(&self) -> RawSocket {
        self.socket.as_raw_socket()
    }
}

impl IntoRawSocket for Socket {
    fn into_raw_socket(self) -> RawSocket {
        self.socket.into_raw_socket()
    }
}

impl FromRawSocket for Socket {
    unsafe fn from_raw_socket(sock: RawSocket) -> Self {
        Self {
            socket: Socket2::from_raw_socket(sock),
        }
    }
}

struct WSASelectGuard<'a, W: AsRawWindow> {
    handle: &'a W,
    socket: &'a Socket2,
}

impl<'a, W: AsRawWindow> WSASelectGuard<'a, W> {
    pub fn new(handle: &'a W, socket: &'a Socket2, event: u32) -> io::Result<Self> {
        syscall_socket(unsafe {
            WSAAsyncSelect(
                socket.as_raw_socket() as _,
                handle.as_raw_window(),
                WM_SOCKET,
                event as _,
            )
        })?;
        Ok(Self { handle, socket })
    }
}

impl<'a, W: AsRawWindow> Drop for WSASelectGuard<'a, W> {
    fn drop(&mut self) {
        syscall_socket(unsafe {
            WSAAsyncSelect(
                self.socket.as_raw_socket() as _,
                self.handle.as_raw_window(),
                0,
                0,
            )
        })
        .unwrap();
        self.socket.set_nonblocking(false).unwrap();
    }
}
