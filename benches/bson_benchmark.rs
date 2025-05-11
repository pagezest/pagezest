use bson::{doc, Bson};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::{self, json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MyData {
    key: String,
    number: i32,
    children: Vec<MyData>,
}

const NUM_CHILDREN: i32 = 2048;

fn benchmark_json(c: &mut Criterion) {
    let json_value = r#"{"key": "value", "number": 42, "children": []}"#;
    let mut json_data: serde_json::Value = serde_json::from_str(black_box(json_value)).unwrap();
    let children = json_data.get_mut("children").and_then(|s| s.as_array_mut()).unwrap();
    for i in 1..NUM_CHILDREN {
        children.push(json!({
            "key": format!("{:?}", i),
            "number": i,
            "children": []
        }));
    }
    let json_value = json_data.to_string();

    c.bench_function("json serialize", |b| {
        b.iter(|| {
            let json_data: serde_json::Value = serde_json::from_str(black_box(&json_value)).unwrap();
            black_box(json_data);
        })
    });

    c.bench_function("json deserialize", |b| {
        b.iter(|| {
            let deserialized: serde_json::Value = serde_json::from_str(black_box(&json_value)).unwrap();
            black_box(deserialized);
        })
    });
}

fn benchmark_bson(c: &mut Criterion) {
    let mut bson_value = bson::to_bson(&doc! { "key": "value", "number": 42, "children": [] }).unwrap();
    let doc = bson_value.as_document_mut().unwrap();
    let children = doc.get_mut("children").unwrap().as_array_mut().unwrap();
    for i in 1..NUM_CHILDREN {
        children.push(Bson::Document(doc! {
            "key": format!("{:?}", i),
            "number": i,
            "children": []
        }));
    }
    let binary_data = bson::to_vec(&bson_value).unwrap();

    c.bench_function("bson serialize", |b| {
        b.iter(|| {
            let bson_data = bson::to_vec(black_box(&bson_value)).unwrap();
            black_box(bson_data);
        })
    });

    c.bench_function("bson deserialize", |b| {
        b.iter(|| {
            let deserialized: Bson = bson::from_slice(black_box(&binary_data)).unwrap();
            let deserialized_doc: bson::Document = bson::from_bson(deserialized).unwrap();
            black_box(deserialized_doc);
        })
    });
}

fn benchmark_bson_struct(c: &mut Criterion) {
    let mut my_data = MyData {
        key: String::from("value"),
        number: 42,
        children: vec![]
    };

    for i in 1..NUM_CHILDREN {
        my_data.children.push(MyData {
            key: format!("{}", i),
            number: i,
            children: vec![]
        });
    }

    c.bench_function("bson struct serialize", |b| {
        b.iter(|| {
            let binary_data = bson::to_vec(black_box(&my_data)).unwrap();
            black_box(binary_data);
        })
    });

    c.bench_function("bson struct deserialize", |b| {
        let binary_data = bson::to_vec(&my_data).unwrap();
        b.iter(|| {
            let deserialized_data: MyData = bson::from_slice(black_box(&binary_data)).unwrap();
            black_box(deserialized_data);
        })
    });
}

criterion_group!(benches, benchmark_json, benchmark_bson, benchmark_bson_struct);
criterion_main!(benches);
