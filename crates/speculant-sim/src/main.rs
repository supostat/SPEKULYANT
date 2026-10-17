//! CLI-симулятор баланса. Стратегии ботов и метрики появятся в фазе `simulator`.

fn main() {
    println!("{}", banner());
}

fn banner() -> String {
    format!(
        "speculant-sim {} — симулятор баланса на ядре speculant-core {}",
        env!("CARGO_PKG_VERSION"),
        speculant_core::VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::banner;

    #[test]
    fn banner_names_the_core_version() {
        assert!(banner().contains(speculant_core::VERSION));
    }
}
