use criterion::{Criterion, black_box, criterion_group, criterion_main};
use wordle_rust::game::WordList;
use wordle_rust::solver::{EntropyCalculator, WordleSolver};

fn benchmark_entropy_calculation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let word_list = rt.block_on(async { WordList::new().await.unwrap() });
    let mut entropy_calc = EntropyCalculator::new();
    let possible_words: Vec<String> = word_list.get_answer_words()[..100].to_vec();

    c.bench_function("entropy_calculation", |b| {
        b.iter(|| entropy_calc.calculate_entropy(black_box("adieu"), black_box(&possible_words)))
    });
}

fn benchmark_best_guess_selection(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("best_guess_selection", |b| {
        b.iter(|| {
            let mut solver = rt.block_on(async { WordleSolver::new().await.unwrap() });
            solver.get_best_guess().unwrap()
        })
    });
}

fn benchmark_guess_filtering(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("guess_filtering", |b| {
        b.iter(|| {
            let mut test_solver = rt.block_on(async { WordleSolver::new().await.unwrap() });
            test_solver
                .add_guess_result(black_box("adieu"), black_box("20100"))
                .unwrap();
        })
    });
}

criterion_group!(
    benches,
    benchmark_entropy_calculation,
    benchmark_best_guess_selection,
    benchmark_guess_filtering
);
criterion_main!(benches);
