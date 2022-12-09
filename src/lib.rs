pub use prost;
use prost::Message;
pub mod codec;
pub mod error;
use error::Error;
use error::Result;

use bytes::BytesMut;
// use std::collections::HashMap;
use std::marker::PhantomData;

use tokio::io::AsyncWriteExt;

pub mod krpc {
    include!(concat!(env!("OUT_DIR"), "/krpc.schema.rs"));
}

// type StreamID = u64;

/// An async client to a RPC server.
pub struct RPCClient {}

#[doc(hidden)]
/// Represents a request that can be submitted to the RPCServer. Library users should not have to
/// use this object directly.
#[derive(Clone)]
pub struct RPCRequest {
    calls: Vec<krpc::ProcedureCall>,
}

/// Represents a procedure call. The type parameter is the type of the value to be extracted from
/// the server's response.
#[derive(Clone)]
pub struct CallHandle<T> {
    proc_call: krpc::ProcedureCall,
    _phantom: PhantomData<T>,
}

// /// A handle to a stream. The type parameter is the type of the value produced by the stream.
// #[derive(Copy, Clone, Debug)]
// pub struct StreamHandle<T> {
//     stream_id: StreamID,
//     _phantom: PhantomData<T>,
// }
//
// /// A collection of updates received from the stream server.
// #[derive(Debug)]
// pub struct StreamUpdate {
//     updates: HashMap<StreamID, krpc::ProcedureResult>,
// }

#[doc(hidden)]
#[macro_export]
macro_rules! batch_call_common {
    ($process_result:expr, $client:expr, ( $( $call:expr ),+ )) => {{
        let mut request = $crate::RPCRequest::new();
        $( request.add_call($call); )+
        match $client.submit_request(request) {
            Err(e) => {
                Err(e)
            }
            Ok(ref mut response) if response.has_error() => {
                Err($crate::error::Error::Request(response.take_error()))
            }
            Ok(ref response) => {
                let mut _i = 0;
                Ok(( $({
                        let result = $call.get_result(&response.results[_i]); _i += 1;
                        $process_result(result)
                },)+ ))
            }
        }
    }};
}

/// Groups calls in a single packet. The return value is a tuple of `Result`s, one for each call.
///
/// # Example:
/// ```rust,ignore
///let client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;
///let (vessel, time) = batch_call!(&client, (
///    &space_center::get_active_vessel(),
///    &space_center::get_ut(),
///))?;
/// ```
#[macro_export]
macro_rules! batch_call {
    ($client:expr, ( $( $call:expr ),+ )) => {
        batch_call_common!(|result| { result }, $client, ( $( $call ),+ ))
    };
    ($client:expr, ( $( $call:expr ),+ , )) => /* retry without last ';' */ {
        batch_call!($client, ( $( $call ),+ ))
    };
}

/// Does the same as `batch_call` but unwraps all values automatically.
#[macro_export]
macro_rules! batch_call_unwrap {
    ($client:expr, ( $( $call:expr ),+ )) => {{
        batch_call_common!(|result: $crate::error::Result<_>| { result.unwrap() }, $client, ( $( $call ),+ ))
    }};
    ($client:expr, ( $( $call:expr ),+ , )) => /* retry without last ';' */ {
        batch_call_unwrap!($client, ( $( $call ),+ ))
    };
}

/// Creates a stream request from a CallHandle. For less verbosity, you can use the `to_stream()`
/// method on `CallHandle`s instead.
///
/// Note that types don't prevent you from chaining multiple `mk_stream`. This will build a stream
/// request of a stream request. Turns out this is accepted by the RPC server and the author of
/// this library confesses he had some fun with this.
// pub fn mk_stream<T: codec::RPCExtractable>(call: &CallHandle<T>) -> CallHandle<StreamHandle<T>> {
//     let arg = krpc::Argument {
//         position: 0,
//         value: call.get_call().encode_to_vec(),
//     };
//
//     let mut arguments = Vec::<krpc::Argument>::new();
//     arguments.push(arg);
//
//     let proc_call = krpc::ProcedureCall {
//         service: "KRPC".to_string(),
//         procedure: "AddStream".to_string(),
//         arguments,
//         ..Default::default()
//     };
//
//     CallHandle::<StreamHandle<T>>::new(proc_call)
// }

impl RPCClient {
    pub async fn connect(client_name: &str, addr: impl tokio::net::ToSocketAddrs) -> Result<Self> {
        let socket = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(Error::Io)?;

        let (mut socket_reader, mut socket_writer) = socket.into_split();

        // Create a connection request
        let conn_req = krpc::ConnectionRequest {
            r#type: krpc::connection_request::Type::Rpc as i32,
            client_name: client_name.to_string(),
            ..Default::default()
        };

        let mut buf = BytesMut::new();
        conn_req.encode(&mut buf).map_err(Error::EncodeError)?;

        socket_writer.write_all(&buf).await.map_err(Error::Io)?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let connection_response =
            codec::read_message::<krpc::ConnectionResponse>(&mut socket_reader).await?;

        println!("Connection response: {:?}", connection_response);

        Ok(Self {})
    }

    // pub async fn request<T: codec::RPCExtractable>(&self, call: &CallHandle<T>) -> Result<T> {
    //     // Make a copy of the call
    //     let call = call.clone();
    //
    //     // Create a request
    //     let mut request = RPCRequest::new();
    //     request.add_call(call);
    //
    //     // Encode the request
    //     let mut buf = BytesMut::new();
    //     request
    //         .build()
    //         .encode(&mut buf)
    //         .map_err(Error::EncodeError)?;
    //
    //     // temporary fake socket
    //     let socket = tokio::net::TcpStream::connect("127.0.0.1:50000")
    //         .await
    //         .map_err(Error::Io)?;
    //
    //     let (mut reader, mut writer) = socket.into_split();
    //     writer.write_all(&buf).await.map_err(Error::Io)?;
    //
    //     let mut read_buffer = BytesMut::new();
    //     let _count: usize = reader.read_buf(&mut read_buffer).await.map_err(Error::Io)?;
    //
    //     let result = krpc::ProcedureResult::decode(read_buffer).map_err(Error::DecodeError)?;
    //     call.get_result(&result)
    // }
}

impl RPCRequest {
    pub fn new() -> Self {
        RPCRequest {
            calls: Vec::<krpc::ProcedureCall>::new(),
        }
    }

    pub fn add_call<T: codec::RPCExtractable>(&mut self, handle: &CallHandle<T>) {
        self.calls.push(handle.get_call().clone())
    }

    fn build(self) -> krpc::Request {
        krpc::Request { calls: self.calls }
    }
}

impl<T> CallHandle<T>
where
    T: codec::RPCExtractable,
{
    pub fn new(proc_call: krpc::ProcedureCall) -> Self {
        CallHandle {
            proc_call,
            _phantom: PhantomData,
        }
    }

    // pub fn to_stream(&self) -> CallHandle<StreamHandle<T>> {
    //     mk_stream(self)
    // }

    pub fn get_result(&self, result: &krpc::ProcedureResult) -> Result<T> {
        codec::extract_result(result)
    }

    fn get_call(&self) -> &krpc::ProcedureCall {
        &self.proc_call
    }
}

// impl<T> StreamHandle<T> {
//     pub fn new(stream_id: StreamID) -> Self {
//         StreamHandle {
//             stream_id,
//             _phantom: PhantomData,
//         }
//     }
//
//     pub fn remove(self) -> CallHandle<()> {
//         use codec::RPCEncodable;
//
//         let arg = krpc::Argument {
//             position: 0,
//             value: self.stream_id.encode_to_bytes().unwrap(),
//         };
//
//         let mut arguments = Vec::<krpc::Argument>::new();
//         arguments.push(arg);
//
//         let proc_call = krpc::ProcedureCall {
//             service: "KRPC".to_string(),
//             procedure: "RemoveStream".to_string(),
//             arguments,
//             ..Default::default()
//         };
//
//         CallHandle::<()>::new(proc_call)
//     }
// }
//
// impl StreamUpdate {
//     pub fn get_result<T>(&self, handle: &StreamHandle<T>) -> Result<T>
//     where
//         T: codec::RPCExtractable,
//     {
//         let result = self
//             .updates
//             .get(&handle.stream_id)
//             .ok_or(Error::NoSuchStream)?;
//         codec::extract_result(&result)
//     }
// }
