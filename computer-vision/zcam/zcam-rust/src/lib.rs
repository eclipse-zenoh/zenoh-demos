//
// Copyright (c) 2017, 2026 ZettaScale Technology
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
//
// Contributors:
//   The Zenoh Team, <zenoh@zettascale.tech>
//

use std::{convert::TryInto, fmt::Display};

use futures::StreamExt;
use opencv::core::{Mat, MatTraitConst, ToInputArray};
use rkyv::{Archive, Deserialize, Serialize};
use zenoh::{bytes::ZBytes, key_expr::KeyExpr, sample::Sample, Session};

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
pub struct RawFrameMeta {
    rows: i32,
    cols: i32,
    typ: i32,
    size: usize,
}

impl RawFrameMeta {
    pub fn new(frame: &Mat) -> zenoh::Result<Self> {
        Ok(Self {
            rows: frame.rows(),
            cols: frame.cols(),
            typ: frame.typ(),
            size: frame.size()?.area() as usize * frame.elem_size()?,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// # Safety
    /// 
    /// The caller must ensure that:
    /// - the data pointer is valid and points to a buffer of the correct size
    /// - the data buffer is not modified while the Mat is in use
    /// - the Mat is not used after the data buffer is deallocated
    pub unsafe fn mat_mut(&self, data: *mut u8) -> Mat {
        unsafe { self._mat(data) }
    }

    /// # Safety
    /// 
    /// The caller must ensure that:
    /// - the data pointer is valid and points to a buffer of the correct size
    /// - the data buffer is not modified while the Mat is in use
    /// - the Mat is not used after the data buffer is deallocated
    pub unsafe fn mat(&self, data: *const u8) -> impl MatTraitConst + ToInputArray {
        unsafe { self._mat(data as *mut u8) }
    }

    unsafe fn _mat(&self, data: *mut u8) -> Mat {
        unsafe {
            Mat::new_rows_cols_with_data_unsafe_def(
                self.rows,
                self.cols,
                self.typ,
                data as *mut std::ffi::c_void,
            )
            .unwrap()
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug)]
pub enum FrameMeta {
    Raw(RawFrameMeta),
    Jpeg(RawFrameMeta),
}

impl FrameMeta {
    pub fn decode(sample: &Sample) -> zenoh::Result<Self> {
        let attachment = sample.attachment().ok_or("Missing attachment")?;
        let attachment_bytes = attachment.to_bytes();
        let meta = rkyv::access::<ArchivedFrameMeta, rkyv::rancor::Error>(&attachment_bytes)?;
        let meta = rkyv::deserialize::<FrameMeta, rkyv::rancor::Error>(meta)?;
        Ok(meta)
    }

    pub fn encode(&self) -> zenoh::Result<ZBytes> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;
        Ok(bytes.as_slice().into())
    }
}

impl Display for FrameMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameMeta::Raw(meta) => write!(
                f,
                "FrameMeta::Raw {{ rows: {}, cols: {}, typ: {} }}",
                meta.rows, meta.cols, meta.typ
            ),
            FrameMeta::Jpeg(meta) => write!(
                f,
                "FrameMeta::Jpeg {{ rows: {}, cols: {}, typ: {} }}",
                meta.rows, meta.cols, meta.typ
            ),
        }
    }
}

pub async fn config_update_loop<'a, TryIntoKeyExpr>(
    session: &Session,
    config_keyexpr: TryIntoKeyExpr,
) where
    TryIntoKeyExpr: TryInto<KeyExpr<'a>>,
    <TryIntoKeyExpr as TryInto<KeyExpr<'a>>>::Error: Into<zenoh::Error>,
{
    // Declare subscriber for configuration updates
    let conf_sub = session.declare_subscriber(config_keyexpr).await.unwrap();

    // Loop to receive and apply config updates

    conf_sub
        .stream()
        .for_each(async |sample| {
            let conf_key = sample.key_expr().as_str().split("/conf/").last().unwrap();
            let conf_val = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
            let _ = session.config().insert_json5(conf_key, &conf_val);
        })
        .await;
}
