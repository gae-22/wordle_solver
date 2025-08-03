use criterion::{Criterion, black_box, criterion_group, criterion_main};
use wordle_rust::{CachedEntropyCalculator, EntropyCalculator, Word};

fn benchmark_entropy_calculation(c: &mut Criterion) {
    let entropy_calc = CachedEntropyCalculator::new();

    // Create sample words for testing
    let possible_words: Vec<Word> = ["apple", "about", "bread", "crane", "drive"]
        .iter()
        .map(|s| Word::from_str(s).unwrap())
        .collect();

    let guess = Word::from_str("adieu").unwrap();

    c.bench_function("entropy_calculation", |b| {
        b.iter(|| entropy_calc.calculate_entropy(black_box(&guess), black_box(&possible_words)))
    });
}

fn benchmark_word_creation(c: &mut Criterion) {
    c.bench_function("word_creation", |b| {
        b.iter(|| {
            let _word = Word::from_str(black_box("adieu")).unwrap();
        })
    });
}

fn benchmark_word_validation(c: &mut Criterion) {
    let words = vec!["apple", "about", "bread", "crane", "drive"];

    c.bench_function("word_validation", |b| {
        b.iter(|| {
            for word_str in &words {
                let _valid = Word::from_str(black_box(word_str)).is_ok();
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_entropy_calculation,
    benchmark_word_creation,
    benchmark_word_validation
);
criterion_main!(benches);
