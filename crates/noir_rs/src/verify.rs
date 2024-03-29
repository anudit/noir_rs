use std::io::Read;

use base64::{engine::general_purpose, Engine};
use bb_rs::barretenberg_api::{
    acir::{
        acir_load_verification_key, acir_verify_proof, delete_acir_composer, get_circuit_sizes,
        new_acir_composer,
    },
    srs::init_srs,
};
use flate2::bufread::GzDecoder;

use crate::netsrs::NetSrs;

pub fn verify(
    circuit_bytecode: String,
    proof: Vec<u8>,
    verification_key: Vec<u8>,
) -> Result<bool, String> {
    let acir_buffer = general_purpose::STANDARD
        .decode(circuit_bytecode)
        .map_err(|e| e.to_string())?;

    let mut decoder = GzDecoder::new(acir_buffer.as_slice());
    let mut acir_buffer_uncompressed = Vec::<u8>::new();
    decoder
        .read_to_end(&mut acir_buffer_uncompressed)
        .map_err(|e| e.to_string())?;

    let circuit_size = unsafe { get_circuit_sizes(&acir_buffer_uncompressed) };
    let log_value = (circuit_size.total as f64).log2().ceil() as u32;
    let subgroup_size = 2u32.pow(log_value);

    let srs = NetSrs::new(subgroup_size + 1);

    Ok(unsafe {
        init_srs(&srs.data, srs.num_points, &srs.g2_data);
        let mut acir_ptr = new_acir_composer(subgroup_size);
        acir_load_verification_key(&mut acir_ptr, &verification_key);
        let result = acir_verify_proof(&mut acir_ptr, &proof);
        delete_acir_composer(acir_ptr);
        result
    })
}
