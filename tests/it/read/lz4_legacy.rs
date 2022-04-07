use parquet2::compression;
use rand::RngCore;
use std::io::Write;

fn lz4_legacy_compress(input: &[u8]) -> std::io::Result<Vec<u8>> {
    let output = Vec::new();
    let mut encoder = lz4::EncoderBuilder::new().build(output)?;
    encoder.write_all(input)?;
    let (compressed, result) = encoder.finish();
    result.map(|_| compressed)
}

#[test]
fn lz4_legacy_decompression() -> parquet2::error::Result<()> {
    let sizes = vec![0, 16, 256, 4096, 8192];
    for size in sizes {
        let mut input = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut input);
        let compressed = lz4_legacy_compress(&input)?;
        let mut output = vec![0; size];
        compression::decompress(compression::Compression::Lz4, &compressed, &mut output)?;
        assert_eq!(input, output);
    }
    Ok(())
}
