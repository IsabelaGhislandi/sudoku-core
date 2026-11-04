use std::time::{Duration, Instant};

use sudoku_core::{Difficulty, GenError, generate};

/// Mede o gerador de Médio. Fica fora da suíte normal porque tempo só faz sentido em
/// build de release: `cargo test --release --test desempenho -- --ignored --nocapture`.
///
/// A meta de 1 s neste PC é uma estimativa para o limite de 5 s no celular, supondo um
/// aparelho até 5 vezes mais lento. Deve ser confirmada em um aparelho real.
#[test]
#[ignore]
fn medio_cabe_na_meta_de_tempo() {
    let mut times: Vec<Duration> = (0..200)
        .map(|seed| {
            let start = Instant::now();
            generate(Difficulty::Medio, seed).unwrap_or_else(|e| panic!("seed {seed}: {e:?}"));
            start.elapsed()
        })
        .collect();
    times.sort();
    let worst = *times.last().unwrap();
    println!("mediana: {:?}, pior: {:?}", times[100], worst);
    assert!(worst <= Duration::from_secs(1), "pior caso: {worst:?}");
}

/// Mede o gerador de Difícil. A meta é por chamada, termine ela com um puzzle ou com
/// `AttemptsExhausted`: as seeds raras que esgotam as tentativas são aceitas e só
/// contadas, porque o Game sorteia outra seed nesses casos.
#[test]
#[ignore]
fn dificil_cabe_na_meta_de_tempo() {
    let mut exhausted = 0;
    let mut times: Vec<Duration> = (0..200)
        .map(|seed| {
            let start = Instant::now();
            match generate(Difficulty::Dificil, seed) {
                Ok(_) => {}
                Err(GenError::AttemptsExhausted(_)) => exhausted += 1,
            }
            start.elapsed()
        })
        .collect();
    times.sort();
    let worst = *times.last().unwrap();
    println!(
        "mediana: {:?}, pior: {:?}, esgotadas: {exhausted}",
        times[100], worst
    );
    assert!(worst <= Duration::from_secs(1), "pior caso: {worst:?}");
}

/// Mede o gerador de Muito Difícil, no molde do Difícil. A meta é por chamada, termine ela
/// com um puzzle ou com `AttemptsExhausted`; as seeds esgotadas são contadas e podem ser no
/// máximo 1 em 20, porque cada uma faz quem joga esperar de novo enquanto o Game sorteia
/// outra seed.
#[test]
#[ignore]
fn muito_dificil_cabe_na_meta_de_tempo() {
    let mut exhausted = 0;
    let mut times: Vec<Duration> = (0..200)
        .map(|seed| {
            let start = Instant::now();
            match generate(Difficulty::MuitoDificil, seed) {
                Ok(_) => {}
                Err(GenError::AttemptsExhausted(_)) => exhausted += 1,
            }
            start.elapsed()
        })
        .collect();
    times.sort();
    let worst = *times.last().unwrap();
    println!(
        "mediana: {:?}, p90: {:?}, pior: {:?}, esgotadas: {exhausted}",
        times[100], times[180], worst
    );
    assert!(worst <= Duration::from_secs(1), "pior caso: {worst:?}");
    assert!(exhausted <= 10, "esgotadas: {exhausted}");
}
