use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_barcode(data: &str) -> Result<JsValue, JsError> {
    let barcode_data = aamva::parse_barcode(data).map_err(|err| JsError::new(&err.to_string()))?;

    Ok(serde_wasm_bindgen::to_value(&barcode_data)?)
}
