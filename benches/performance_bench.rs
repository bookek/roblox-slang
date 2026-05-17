use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use roblox_slang::generator::{csv, luau};
use roblox_slang::utils::flatten;
use serde_json::Value;
use std::collections::HashMap;

// Performance benchmarks for Roblox Slang
//
// Benchmarks cover:
// - JSON parsing (1K, 5K, 10K translations)
// - YAML parsing
// - Luau code generation
// - CSV generation
// - Flatten operations
//
// Run with: cargo bench

// Helper Functions
/// Generate test translations with nested structure
fn generate_nested_translations(count: usize) -> Value {
    let mut map = serde_json::Map::new();

    for i in 0..count {
        let category = format!("category_{}", i / 100);
        let subcategory = format!("subcategory_{}", i / 10);
        let key = format!("key_{}", i);
        let value = format!("Translation value {}", i);

        if !map.contains_key(&category) {
            map.insert(category.clone(), Value::Object(serde_json::Map::new()));
        }

        if let Some(Value::Object(cat_map)) = map.get_mut(&category) {
            if !cat_map.contains_key(&subcategory) {
                cat_map.insert(subcategory.clone(), Value::Object(serde_json::Map::new()));
            }

            if let Some(Value::Object(subcat_map)) = cat_map.get_mut(&subcategory) {
                subcat_map.insert(key, Value::String(value));
            }
        }
    }

    Value::Object(map)
}

/// Generate flat translations
fn generate_flat_translations(count: usize) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for i in 0..count {
        let key = format!("category_{}.subcategory_{}.key_{}", i / 100, i / 10, i);
        let value = format!("Translation value {}", i);
        map.insert(key, value);
    }

    map
}

// JSON Parsing Benchmarks

fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    for size in [100, 1000, 5000, 10000].iter() {
        let json = generate_nested_translations(*size);
        let json_str = serde_json::to_string(&json).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_translations", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let parsed: Value = serde_json::from_str(black_box(&json_str)).unwrap();
                    black_box(parsed);
                });
            },
        );
    }

    group.finish();
}

// YAML Parsing Benchmarks

fn bench_yaml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("yaml_parsing");

    for size in [100, 1000, 5000].iter() {
        let json = generate_nested_translations(*size);
        let yaml_str = serde_yaml::to_string(&json).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_translations", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let parsed: Value = serde_yaml::from_str(black_box(&yaml_str)).unwrap();
                    black_box(parsed);
                });
            },
        );
    }

    group.finish();
}

// Flatten Benchmarks

fn bench_flatten_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("flatten");

    for size in [100, 1000, 5000, 10000].iter() {
        let json = generate_nested_translations(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_translations", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let flattened = flatten::flatten_json(black_box(&json), String::new());
                    black_box(flattened);
                });
            },
        );
    }

    group.finish();
}

// Luau Code Generation Benchmarks

fn bench_luau_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("luau_generation");

    for size in [100, 1000, 5000, 10000].iter() {
        let flat = generate_flat_translations(*size);
        let translations: Vec<_> = flat
            .iter()
            .map(|(k, v)| roblox_slang::parser::types::Translation {
                key: k.clone(),
                value: v.clone(),
                locale: "en".to_string(),
                context: None,
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_translations", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let code = luau::generate_luau(black_box(&translations), "en").unwrap();
                    black_box(code);
                });
            },
        );
    }

    group.finish();
}

// CSV Generation Benchmarks

fn bench_csv_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_generation");

    for size in [100, 1000, 5000, 10000].iter() {
        let flat = generate_flat_translations(*size);
        let translations: Vec<_> = flat
            .iter()
            .map(|(k, v)| roblox_slang::parser::types::Translation {
                key: k.clone(),
                value: v.clone(),
                locale: "en".to_string(),
                context: None,
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_translations", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let csv_data = csv::generate_csv(
                        black_box(&translations),
                        "en",
                        &["en".to_string(), "id".to_string()],
                    )
                    .unwrap();
                    black_box(csv_data);
                });
            },
        );
    }

    group.finish();
}

// End-to-End Build Benchmarks

fn bench_complete_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_build");
    group.sample_size(10); // Fewer samples for slower benchmarks

    for size in [100, 1000, 5000].iter() {
        let json = generate_nested_translations(*size);
        let json_str = serde_json::to_string(&json).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_translations", size)),
            size,
            |b, _| {
                b.iter(|| {
                    // Parse JSON
                    let parsed: Value = serde_json::from_str(&json_str).unwrap();

                    // Flatten
                    let flattened = flatten::flatten_json(&parsed, String::new());

                    // Convert to translations
                    let translations: Vec<_> = flattened
                        .iter()
                        .map(|(k, v)| roblox_slang::parser::types::Translation {
                            key: k.clone(),
                            value: v.clone(),
                            locale: "en".to_string(),
                            context: None,
                        })
                        .collect();

                    // Generate Luau
                    let _luau = luau::generate_luau(&translations, "en").unwrap();

                    // Generate CSV
                    let _csv = csv::generate_csv(&translations, "en", &["en".to_string()]).unwrap();
                });
            },
        );
    }

    group.finish();
}

// String Operations Benchmarks

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // Benchmark key sanitization
    group.bench_function("sanitize_key", |b| {
        let key = "ui.buttons.buy_now.confirm";
        b.iter(|| {
            let sanitized = key.replace('.', "_");
            black_box(sanitized);
        });
    });

    // Benchmark parameter detection
    group.bench_function("detect_parameters", |b| {
        let text = "Hello {name}, you have {count} messages";
        b.iter(|| {
            let params: Vec<_> = text
                .split('{')
                .skip(1)
                .filter_map(|s| s.split('}').next())
                .collect();
            black_box(params);
        });
    });

    // Benchmark case conversion
    group.bench_function("snake_to_camel", |b| {
        let snake = "ui_button_buy_now";
        b.iter(|| {
            let camel = heck::AsLowerCamelCase(snake).to_string();
            black_box(camel);
        });
    });

    group.finish();
}

// Benchmark Groups

criterion_group!(
    benches,
    bench_json_parsing,
    bench_yaml_parsing,
    bench_flatten_json,
    bench_luau_generation,
    bench_csv_generation,
    bench_complete_build,
    bench_string_operations,
);

criterion_main!(benches);
