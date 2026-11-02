use std::time::{Duration, Instant};

use sudoku_core::{Difficulty, generate};

/// Mede o gerador de Médio. Fica fora da suíte normal porque tempo só faz sentido em
/// build de release: `cargo test --release --test desempenho -- --ignored --nocapture`.
///
/// A meta de 1 s neste PC é uma estimativa para o limite de 5 s no celular, supondo um
/// aparelho até 5 vezes mais lento. Deve ser confirmada em um aparelho real.
#[test]
#[ignore]
fn medio_cabe_na_meta_de_tempo() {
    let mut tempos: Vec<Duration> = (0..200)
        .map(|seed| {
            let inicio = Instant::now();
            generate(Difficulty::Medio, seed).unwrap_or_else(|e| panic!("seed {seed}: {e:?}"));
            inicio.elapsed()
        })
        .collect();
    tempos.sort();
    let pior = *tempos.last().unwrap();
    println!("mediana: {:?}, pior: {:?}", tempos[100], pior);
    assert!(pior <= Duration::from_secs(1), "pior caso: {pior:?}");
}
