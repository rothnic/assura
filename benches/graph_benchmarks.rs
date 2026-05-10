use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::TempDir;

use assura::intelligence::GraphBuilder;

fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    for i in 0..100 {
        let dir_path = base_path.join(format!("dir{}", i));
        std::fs::create_dir(&dir_path).unwrap();

        for j in 0..10 {
            let file_path = dir_path.join(format!("file{}.rs", j));
            std::fs::write(&file_path, format!("// Test file {}", j)).unwrap();
        }
    }

    temp_dir
}

fn bench_graph_construction(c: &mut Criterion) {
    c.bench_function("graph_construction", |b| {
        b.iter_with_setup(create_test_directory, |temp_dir| {
            let graph = GraphBuilder::new(temp_dir.path()).build().unwrap();
            black_box(graph);
        })
    });
}

fn bench_parallel_graph_construction(c: &mut Criterion) {
    c.bench_function("parallel_graph_construction", |b| {
        b.iter_with_setup(create_test_directory, |temp_dir| {
            let graph = GraphBuilder::new(temp_dir.path()).build_parallel().unwrap();
            black_box(graph);
        })
    });
}

fn bench_graph_queries(c: &mut Criterion) {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path()).build().unwrap();

    c.bench_function("graph_query_by_type", |b| {
        b.iter(|| {
            let query = assura::intelligence::GraphQuery::new(&graph);
            let result = query.find_by_type(assura::intelligence::NodeType::File);
            black_box(result);
        })
    });

    c.bench_function("graph_search_by_pattern", |b| {
        b.iter(|| {
            let query = assura::intelligence::GraphQuery::new(&graph);
            let result = query.search_by_pattern("file");
            black_box(result);
        })
    });
}

criterion_group!(
    benches,
    bench_graph_construction,
    bench_parallel_graph_construction,
    bench_graph_queries
);
criterion_main!(benches);
