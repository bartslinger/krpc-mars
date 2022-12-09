use crate::error::Error;
use crate::krpc;

use prost;
use prost::Message;

use bytes::BytesMut;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use tokio::io::AsyncReadExt;

pub trait RPCExtractable: Sized {
    fn extract_value(input: &mut BytesMut) -> Result<Self, prost::DecodeError>;
}

impl RPCExtractable for bool {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for () {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for f64 {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for f32 {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for u64 {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for u32 {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for i64 {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for i32 {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

impl RPCExtractable for String {
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        Message::decode(&mut input)
    }
}

// impl<T> RPCExtractable for super::StreamHandle<T>
// where
//     T: RPCExtractable,
// {
//     fn extract_value(
//         input: &mut Vec<u8>,
//     ) -> Result<Self, prost::DecodeError> {
//         let mut stream = krpc::Stream::new();
//         stream.merge_from(input)?;
//         Ok(super::StreamHandle::new(stream.id))
//     }
// }

impl<T> RPCExtractable for Vec<T>
where
    T: RPCExtractable,
{
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        let m = krpc::List::decode(input)?;

        let mut v = Vec::with_capacity(m.items.len());
        for item in m.items {
            let mut i = BytesMut::from(item.as_slice());
            v.push(RPCExtractable::extract_value(&mut i)?);
        }

        Ok(v)
    }
}

impl<T> RPCExtractable for HashSet<T>
where
    T: RPCExtractable + Hash + Eq,
{
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        let m = krpc::Set::decode(input)?;

        let mut s = HashSet::with_capacity(m.items.len());
        for item in m.items {
            let mut i = BytesMut::from(item.as_slice());
            s.insert(RPCExtractable::extract_value(&mut i)?);
        }

        Ok(s)
    }
}

impl<T, U> RPCExtractable for HashMap<T, U>
where
    T: RPCExtractable + Hash + Eq,
    U: RPCExtractable,
{
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        let m = krpc::Dictionary::decode(input)?;

        let mut h = HashMap::with_capacity(m.entries.len());
        for entry in m.entries {
            let mut i_k = BytesMut::from(entry.key.as_slice());
            let mut i_v = BytesMut::from(entry.value.as_slice());
            let key = RPCExtractable::extract_value(&mut i_k)?;
            let val = RPCExtractable::extract_value(&mut i_v)?;
            h.insert(key, val);
        }

        Ok(h)
    }
}

impl<T, U> RPCExtractable for (T, U)
where
    T: RPCExtractable,
    U: RPCExtractable,
{
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        let l = krpc::Tuple::decode(input)?;
        let t = RPCExtractable::extract_value(&mut BytesMut::from(l.items[0].as_slice()))?;
        let u = RPCExtractable::extract_value(&mut BytesMut::from(l.items[1].as_slice()))?;
        Ok((t, u))
    }
}

impl<T, U, V> RPCExtractable for (T, U, V)
where
    T: RPCExtractable,
    U: RPCExtractable,
    V: RPCExtractable,
{
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        let l = krpc::Tuple::decode(input)?;
        let t = RPCExtractable::extract_value(&mut BytesMut::from(l.items[0].as_slice()))?;
        let u = RPCExtractable::extract_value(&mut BytesMut::from(l.items[1].as_slice()))?;
        let v = RPCExtractable::extract_value(&mut BytesMut::from(l.items[2].as_slice()))?;
        Ok((t, u, v))
    }
}

impl<T, U, V, W> RPCExtractable for (T, U, V, W)
where
    T: RPCExtractable,
    U: RPCExtractable,
    V: RPCExtractable,
    W: RPCExtractable,
{
    fn extract_value(mut input: &mut BytesMut) -> Result<Self, prost::DecodeError> {
        let l = krpc::Tuple::decode(input)?;
        let t = RPCExtractable::extract_value(&mut BytesMut::from(l.items[0].as_slice()))?;
        let u = RPCExtractable::extract_value(&mut BytesMut::from(l.items[1].as_slice()))?;
        let v = RPCExtractable::extract_value(&mut BytesMut::from(l.items[2].as_slice()))?;
        let w = RPCExtractable::extract_value(&mut BytesMut::from(l.items[3].as_slice()))?;
        Ok((t, u, v, w))
    }
}

pub trait RPCEncodable {
    fn encode(&self, output: &mut BytesMut) -> Result<(), prost::EncodeError>;
    fn encode_to_bytes(&self) -> Result<Vec<u8>, prost::EncodeError> {
        let mut bytes = BytesMut::new();
        {
            self.encode(&mut bytes)?;
        }
        Ok(bytes.to_vec())
    }
}

impl RPCEncodable for bool {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl RPCEncodable for f64 {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl RPCEncodable for f32 {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl RPCEncodable for u32 {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl RPCEncodable for i32 {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl RPCEncodable for u64 {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl RPCEncodable for i64 {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl RPCEncodable for String {
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        Message::encode(self, &mut output)
    }
}

impl<T> RPCEncodable for Vec<T>
where
    T: RPCEncodable,
{
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        let mut v = Vec::<Vec<u8>>::new();
        for e in self {
            v.push(e.encode_to_bytes()?);
        }

        let l = krpc::List { items: v };
        l.encode(&mut output)?;

        Ok(())
    }
}

impl<T, U> RPCEncodable for (T, U)
where
    T: RPCEncodable,
    U: RPCEncodable,
{
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        let &(ref t, ref u) = self;
        let tuple = krpc::Tuple {
            items: vec![t.encode_to_bytes()?, u.encode_to_bytes()?],
        };
        tuple.encode(&mut output)?;
        Ok(())
    }
}

impl<T, U, V> RPCEncodable for (T, U, V)
where
    T: RPCEncodable,
    U: RPCEncodable,
    V: RPCEncodable,
{
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        let &(ref t, ref u, ref v) = self;
        let tuple = krpc::Tuple {
            items: vec![
                t.encode_to_bytes()?,
                u.encode_to_bytes()?,
                v.encode_to_bytes()?,
            ],
        };
        tuple.encode(&mut output)?;
        Ok(())
    }
}

impl<T, U, V, W> RPCEncodable for (T, U, V, W)
where
    T: RPCEncodable,
    U: RPCEncodable,
    V: RPCEncodable,
    W: RPCEncodable,
{
    fn encode(&self, mut output: &mut BytesMut) -> Result<(), prost::EncodeError> {
        let &(ref t, ref u, ref v, ref w) = self;
        let tuple = krpc::Tuple {
            items: vec![
                t.encode_to_bytes()?,
                u.encode_to_bytes()?,
                v.encode_to_bytes()?,
                w.encode_to_bytes()?,
            ],
        };
        tuple.encode(&mut output)?;
        Ok(())
    }
}

pub async fn read_message<M>(socket_reader: &mut tokio::net::tcp::OwnedReadHalf) -> Result<M, Error>
where
    M: prost::Message + Default + std::fmt::Debug,
{
    let mut buf = BytesMut::new();
    socket_reader.read_buf(&mut buf).await.map_err(Error::Io)?;
    M::decode(buf).map_err(Error::DecodeError)
}

pub fn extract_result<T>(proc_result: &krpc::ProcedureResult) -> Result<T, Error>
where
    T: RPCExtractable,
{
    let proc_result = proc_result.clone();
    if let Some(error) = proc_result.error {
        Err(Error::Procedure(error))
    } else {
        let mut input = BytesMut::from(proc_result.value.as_slice());
        RPCExtractable::extract_value(&mut input).map_err(Error::DecodeError)
    }
}
