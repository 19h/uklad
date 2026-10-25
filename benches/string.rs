use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use uklad::{Decode, Encode};

#[derive(Serialize, Deserialize, Default, Encode, Decode)]
pub struct MyStruct {
    pub v: Vec<String>,
    pub string: String,
    pub number: usize,
}

impl MyStruct {
    #[inline]
    pub fn new(v: Vec<String>, string: String, number: usize) -> Self {
        Self { v, string, number }
    }
}

fn build_data(size: usize) -> Vec<MyStruct> {
    (0..size)
        .map(|i| {
            let vec: Vec<String> = (0..i).map(|i| i.to_string().repeat(100)).collect();
            MyStruct::new(vec, size.to_string(), size)
        })
        .collect()
}

fn index_item_decode(c: &mut Criterion) {
    let data = build_data(100);

    let config = uklad::config::standard();
    c.bench_function("encode (standard)", |b| {
        b.iter(|| {
            let _ = black_box(uklad::encode_to_vec(black_box(&data), config)).unwrap();
        });
    });

    let config = uklad::config::legacy();
    c.bench_function("encode (legacy)", |b| {
        b.iter(|| {
            let _ = black_box(uklad::encode_to_vec(black_box(&data), config)).unwrap();
        });
    });

    let encoded = uklad::encode_to_vec(&data, config).unwrap();

    c.bench_function("decode (legacy)", |b| {
        b.iter(|| {
            let _: (Vec<MyStruct>, _) =
                black_box(uklad::decode_from_slice(black_box(&encoded), config)).unwrap();
        });
    });
}

criterion_group!(benches, index_item_decode);
criterion_main!(benches);
