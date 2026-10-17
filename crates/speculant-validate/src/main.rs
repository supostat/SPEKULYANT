//! CLI-валидатор контента. Схема и проверки появятся в фазе `content-format`.

fn main() {
    println!("{}", banner());
}

fn banner() -> String {
    format!(
        "speculant-validate {} — валидатор контента на ядре speculant-core {}",
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
