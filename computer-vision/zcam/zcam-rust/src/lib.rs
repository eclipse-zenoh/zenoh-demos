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

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
pub struct RawFrameMeta {
    rows: u16,
    cols: u16,
    typ: u8,
}

impl RawFrameMeta {
pub fn new(frame: &Mat) -> Self {
        Self {
            rows: frame.rows() as u16,
            cols: frame.cols() as u16,
            typ: frame.typ() as u8,
        }
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
    Jpeg,
}

impl Display for FrameMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameMeta::Raw(meta) => write!(
                f,
                "FrameMeta::Raw {{ rows: {}, cols: {}, typ: {} }}",
                meta.rows, meta.cols, meta.typ
            ),
            FrameMeta::Jpeg => write!(f, "FrameMeta::Jpeg"),
        }
    }
}
