//! Send record(s) into Fluentd.

use std::borrow::{Borrow, Cow};
use std::net::ToSocketAddrs;
use std::net;
use std::sync::{
    Mutex,
    Arc,
};
use std::io::Write;
use record::Record;
#[cfg(not(feature = "time-as-integer"))]
use event_record::EventRecord;
use retry_conf::RetryConf;
use forwardable::forward::Forward;
use serde::ser::Serialize;
use serde_json;
use rmp_serde::encode::Serializer;
use error::FluentError;

#[derive(Debug, Clone)]
pub struct Fluent<'a, A>
where
    A: ToSocketAddrs,
{
    addr: A,
    conn: Option<Arc<Mutex<net::TcpStream>>>,
    tag: Cow<'a, str>,
    conf: RetryConf,
}

#[cfg(feature = "time-as-integer")]
type MsgPackSendType<T> = Record<T>;
#[cfg(not(feature = "time-as-integer"))]
type MsgPackSendType<T> = EventRecord<T>;

impl<'a, A: ToSocketAddrs> Fluent<'a, A> {
    /// Create Fluent type.
    ///
    /// ### Usage
    ///
    /// ```
    /// use fruently::fluent::Fluent;
    /// let fruently_with_str_tag = Fluent::new("127.0.0.1:24224", "test");
    /// let fruently_with_string_tag = Fluent::new("127.0.0.1:24224", "test".to_string());
    /// ```
    pub fn new<T>(addr: A, tag: T, conn: Option<Arc<Mutex<net::TcpStream>>>) -> Fluent<'a, A>
    where
        T: Into<Cow<'a, str>>,
    {
        //let mut stream = net::TcpStream::connect(addr)?;
        Fluent {
            addr,
            conn,
            tag: tag.into(),
            conf: RetryConf::new(),
        }
    }

    pub fn new_with_conf<T>(addr: A, tag: T, conf: RetryConf, conn: Option<Arc<Mutex<net::TcpStream>>>) -> Fluent<'a, A>
    where
        T: Into<Cow<'a, str>>,
    {
        let stream = net::TcpStream::connect(addr.borrow()).ok()
            .map(|stream| {
                Arc::new(Mutex::new(stream))
            });
        Fluent {
            addr,
            conn: stream,
            tag: tag.into(),
            conf,
        }
    }

    #[doc(hidden)]
    pub fn get_addr(&self) -> &A {
        self.addr.borrow()
    }

    #[doc(hidden)]
    pub fn get_tag(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(&self.tag)
    }

    #[doc(hidden)]
    pub fn get_conf(&self) -> Cow<RetryConf> {
        Cow::Borrowed(&self.conf)
    }

    pub fn get_conn(&self) -> Option<Arc<Mutex<net::TcpStream>>> {
        self.conn.clone()
    }

    #[doc(hidden)]
    /// For internal usage.
    pub fn closure_send_as_json<T: Serialize>(addr: &A, record: &Record<T>, conn: Option<Arc<Mutex<net::TcpStream>>>) -> Result<(), FluentError> {

        if let Some(conn_mutex) = conn {
            let mut stream = conn_mutex.lock().unwrap();
            let message = serde_json::to_string(&record)?;
            let result = stream.write(&message.into_bytes());
            drop(stream);

            match result {
                Ok(_) => Ok(()),
                Err(v) => Err(From::from(v)),
            }
        } else {
            let mut stream = net::TcpStream::connect(addr)?;
            let message = serde_json::to_string(&record)?;
            let result = stream.write(&message.into_bytes());
            drop(stream);

            match result {
                Ok(_) => Ok(()),
                Err(v) => Err(From::from(v)),
            }
        }
        //let mut stream = net::TcpStream::connect(addr)?;
        /*let message = serde_json::to_string(&record)?;
        let result = stream.write(&message.into_bytes());
        drop(stream);

        match result {
            Ok(_) => Ok(()),
            Err(v) => Err(From::from(v)),
        }*/
    }

    #[doc(hidden)]
    /// For internal usage.
    pub fn closure_send_as_msgpack<T: Serialize>(addr: &A, record: &MsgPackSendType<T>) -> Result<(), FluentError> {
        let mut stream = net::TcpStream::connect(addr)?;
        let result = record.serialize(&mut Serializer::new(&mut stream));

        match result {
            Ok(_) => Ok(()),
            Err(v) => Err(From::from(v)),
        }
    }

    #[doc(hidden)]
    /// For internal usage.
    pub fn closure_send_as_forward<T: Serialize>(addr: &A, forward: &Forward<T>) -> Result<(), FluentError> {
        let mut stream = net::TcpStream::connect(addr)?;
        let result = forward.serialize(&mut Serializer::new(&mut stream));

        match result {
            Ok(_) => Ok(()),
            Err(v) => Err(From::from(v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use retry_conf::RetryConf;
    use std::borrow::Cow;

    #[test]
    fn create_fruently() {
        let fruently = Fluent::new("127.0.0.1:24224", "test");
        let expected = Fluent {
            conn: None,
            addr: "127.0.0.1:24224",
            tag: Cow::Borrowed("test"),
            conf: RetryConf::new(),
        };
        assert_eq!(expected, fruently);
    }
}
