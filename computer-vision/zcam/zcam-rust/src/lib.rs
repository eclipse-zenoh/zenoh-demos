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

use std::fmt::Display;

use opencv::core::{Mat, MatTraitConst, ToInputArray};
use rkyv::{Archive, Deserialize, Serialize};
use zenoh::sample::Sample;

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
pub struct RawFrameMeta {
    rows: u16,
    cols: u16,
    typ: u8,
    size: usize,
}

impl RawFrameMeta {
    pub fn new(frame: &Mat) -> zenoh::Result<Self> {
        Ok(Self {
            rows: frame.rows() as u16,
            cols: frame.cols() as u16,
            typ: frame.typ() as u8,
            size: frame.size()?.area() as usize * frame.elem_size()? as usize,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub unsafe fn mat_mut(&self, data: *mut u8) -> Mat {
        unsafe { self._mat(data) }
    }

    pub unsafe fn mat(&self, data: *const u8) -> impl MatTraitConst + ToInputArray {
        unsafe { self._mat(data as *mut u8) }
    }

    unsafe fn _mat(&self, data: *mut u8) -> Mat {
        unsafe {
            Mat::new_rows_cols_with_data_unsafe_def(
                self.rows as i32,
                self.cols as i32,
                self.typ as i32,
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
    pub fn decode_from_sample(sample: &Sample) -> zenoh::Result<Self> {
        let attachment = sample.attachment().ok_or("Missing attachment")?;
        let attachment_bytes = attachment.to_bytes();
        let meta = rkyv::access::<ArchivedFrameMeta, rkyv::rancor::Error>(&attachment_bytes)?;
        let meta = rkyv::deserialize::<FrameMeta, rkyv::rancor::Error>(meta)?;
        Ok(meta)
    }

    pub fn encode_to_sample(&self, sample: &mut Sample) -> zenoh::Result<()> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;
        sample.set_attachment(Some(bytes.as_slice().into()));
        Ok(())
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
