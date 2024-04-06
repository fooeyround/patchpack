#![feature(test)]

extern crate test;

use std::io::{Read, Result};
use test::Bencher;

use liblzma::{decode_all, encode_all};

use patchpack;

fn compress_bench(x: &[u8], b: &mut Bencher) {
    b.iter(|| encode_all(x, 9).unwrap());
}

fn decompress_bench(x: &[u8], b: &mut Bencher) {
    b.iter(|| decode_all(x).unwrap());
}
