use candle_core::Device;

#[test]
fn encoder_from_safetensors_rejects_missing_weights() {
    let dir = tempfile::tempdir().expect("tempdir");
    let weights = dir.path().join("model.safetensors");
    let config = dir.path().join("config.json");
    let result = crate::encoder::Encoder::from_safetensors(&weights, &config, &Device::Cpu);
    match result {
        Ok(_) => panic!("missing files must error, not panic"),
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("encoder config read") || msg.contains("backend error"),
                "unexpected error: {msg}"
            );
        }
    }
}
