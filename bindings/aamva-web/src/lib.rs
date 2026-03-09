use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    js_name = "parseBarcode",
    return_description = "barcode data parsed into raw files",
    unchecked_return_type = "Data"
)]
pub fn parse_barcode(
    #[wasm_bindgen(param_description = "barcode data")] data: &str,
) -> Result<JsValue, JsError> {
    let barcode_data = aamva::parse_barcode(data).map_err(|err| JsError::new(&err.to_string()))?;

    Ok(serde_wasm_bindgen::to_value(&barcode_data)?)
}

#[wasm_bindgen(
    js_name = "decodeBarcode",
    return_description = "barcode data parsed into a standard representation"
)]
pub fn decode_barcode(
    #[wasm_bindgen(param_description = "barcode data")] data: &str,
) -> Result<aamva::DecodedData, JsError> {
    let barcode_data = aamva::parse_barcode(data).map_err(|err| JsError::new(&err.to_string()))?;
    let decoded_data: aamva::DecodedData = barcode_data.into();

    Ok(decoded_data)
}
