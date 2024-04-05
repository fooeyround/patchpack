#![feature(test)]

extern crate test;

use std::io::Read;
use test::Bencher;




fn compress_bench(x: &[u8], b: &mut Bencher) {
    b.iter(|| {
        let mut compressed: Vec<u8> = Vec::new();
        lzma_rs::lzma_compress(&mut std::io::BufReader::new(x), &mut compressed).unwrap();
        compressed
    });
}