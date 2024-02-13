use std::{
    future::Future,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    ptr::{null, null_mut},
};

use compio_buf::BufResult;
use either::Either;
use socket2::SockAddr;
use widestring::U16CString;
use windows_sys::Win32::Networking::WinSock::{
    FreeAddrInfoExW, GetAddrInfoExW, ADDRINFOEXW, AF_UNSPEC, IPPROTO_TCP, NS_ALL, SOCK_STREAM,
};

use crate::{winsock_result, with_gai};

#[allow(async_fn_in_trait)]
pub trait ToSocketAddrsAsync {
    /// See [`std::net::ToSocketAddrs::Iter`].
    type Iter: Iterator<Item = SocketAddr>;

    /// See [`std::net::ToSocketAddrs::to_socket_addrs`].
    async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter>;
}

macro_rules! impl_to_socket_addrs_async {
    ($($t:ty),* $(,)?) => {
        $(
            impl ToSocketAddrsAsync for $t {
                type Iter = std::iter::Once<SocketAddr>;

                async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter> {
                    Ok(std::iter::once(SocketAddr::from(*self)))
                }
            }
        )*
    }
}

impl_to_socket_addrs_async![
    SocketAddr,
    SocketAddrV4,
    SocketAddrV6,
    (IpAddr, u16),
    (Ipv4Addr, u16),
    (Ipv6Addr, u16),
];

impl ToSocketAddrsAsync for (&str, u16) {
    type Iter = Either<std::iter::Once<SocketAddr>, std::vec::IntoIter<SocketAddr>>;

    async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter> {
        let (host, port) = self;
        if let Ok(addr) = host.parse::<Ipv4Addr>() {
            return Ok(Either::Left(std::iter::once(SocketAddr::from((
                addr, *port,
            )))));
        }
        if let Ok(addr) = host.parse::<Ipv6Addr>() {
            return Ok(Either::Left(std::iter::once(SocketAddr::from((
                addr, *port,
            )))));
        }

        resolve_sock_addrs(host, *port).await.map(Either::Right)
    }
}

impl ToSocketAddrsAsync for (String, u16) {
    type Iter = <(&'static str, u16) as ToSocketAddrsAsync>::Iter;

    async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter> {
        (&*self.0, self.1).to_socket_addrs_async().await
    }
}

impl ToSocketAddrsAsync for str {
    type Iter = <(&'static str, u16) as ToSocketAddrsAsync>::Iter;

    async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter> {
        if let Ok(addr) = self.parse::<SocketAddr>() {
            return Ok(Either::Left(std::iter::once(addr)));
        }

        let (host, port_str) = self
            .rsplit_once(':')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid socket address"))?;
        let port: u16 = port_str
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid port value"))?;
        (host, port).to_socket_addrs_async().await
    }
}

impl ToSocketAddrsAsync for String {
    type Iter = <(&'static str, u16) as ToSocketAddrsAsync>::Iter;

    async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter> {
        self.as_str().to_socket_addrs_async().await
    }
}

impl<'a> ToSocketAddrsAsync for &'a [SocketAddr] {
    type Iter = std::iter::Copied<std::slice::Iter<'a, SocketAddr>>;

    async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().copied())
    }
}

impl<T: ToSocketAddrsAsync + ?Sized> ToSocketAddrsAsync for &T {
    type Iter = T::Iter;

    async fn to_socket_addrs_async(&self) -> io::Result<Self::Iter> {
        (**self).to_socket_addrs_async().await
    }
}

pub(crate) async fn each_addr<T, F: Future<Output = io::Result<T>>>(
    addr: impl ToSocketAddrsAsync,
    f: impl Fn(SocketAddr) -> F,
) -> io::Result<T> {
    let addrs = addr.to_socket_addrs_async().await?;
    let mut last_err = None;
    for addr in addrs {
        match f(addr).await {
            Ok(l) => return Ok(l),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "could not resolve to any addresses",
        )
    }))
}

async fn resolve_sock_addrs(host: &str, port: u16) -> io::Result<std::vec::IntoIter<SocketAddr>> {
    struct InfoGuard {
        result: *mut ADDRINFOEXW,
    }

    impl InfoGuard {
        pub fn new() -> Self {
            Self { result: null_mut() }
        }

        pub unsafe fn addrs(&self, port: u16) -> io::Result<std::vec::IntoIter<SocketAddr>> {
            let mut addrs = vec![];
            let mut result = self.result;
            while let Some(info) = unsafe { result.as_ref() } {
                let addr = unsafe {
                    SockAddr::try_init(|buffer, len| {
                        std::slice::from_raw_parts_mut::<u8>(buffer.cast(), info.ai_addrlen as _)
                            .copy_from_slice(std::slice::from_raw_parts::<u8>(
                                info.ai_addr.cast(),
                                info.ai_addrlen as _,
                            ));
                        *len = info.ai_addrlen as _;
                        Ok(())
                    })
                }
                // it is always Ok
                .unwrap()
                .1;
                if let Some(mut addr) = addr.as_socket() {
                    addr.set_port(port);
                    addrs.push(addr)
                }
                result = info.ai_next;
            }
            Ok(addrs.into_iter())
        }
    }

    impl Drop for InfoGuard {
        fn drop(&mut self) {
            if !self.result.is_null() {
                unsafe { FreeAddrInfoExW(self.result) }
            }
        }
    }

    let name = U16CString::from_str(host)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid host name"))?;

    let BufResult(res, info) = with_gai(
        |optr, callback, info| {
            let mut hints: ADDRINFOEXW = unsafe { std::mem::zeroed() };
            hints.ai_family = AF_UNSPEC as _;
            hints.ai_socktype = SOCK_STREAM;
            hints.ai_protocol = IPPROTO_TCP;

            let res = unsafe {
                GetAddrInfoExW(
                    name.as_ptr(),
                    null(),
                    NS_ALL,
                    null(),
                    &hints,
                    &mut info.result,
                    null(),
                    optr,
                    callback,
                    null_mut(),
                )
            };
            winsock_result(res)
        },
        InfoGuard::new(),
    )
    .await;
    res?;
    unsafe { info.addrs(port) }
}
