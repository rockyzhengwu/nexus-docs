#![deny(clippy::all)]

use onnx_ocr::ocr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn plus_100(input: u32) -> u32 {
  input + 100
}

#[napi(object)]
pub struct OcrResult {
  pub bbox: Vec<(u32, u32)>,
  pub content: String,
}

#[napi]
pub fn ocr(buffer: Buffer) -> Vec<OcrResult> {
  let bytes: Vec<u8> = buffer.into();
  let predict_result = ocr::ocr(bytes.as_slice()).unwrap();
  let mut res = Vec::new();
  for pr in predict_result {
    let bbox = vec![
      (pr.bbox.tl.x, pr.bbox.tl.y),
      (pr.bbox.tr.x, pr.bbox.tr.y),
      (pr.bbox.dr.x, pr.bbox.dr.y),
      (pr.bbox.dl.x, pr.bbox.dl.y),
    ];
    let or = OcrResult {
      bbox,
      content: pr.content,
    };
    res.push(or)
  }

  return res;
}
