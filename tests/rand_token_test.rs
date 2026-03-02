use rand::distr::Alphanumeric;
use rand::RngExt;

#[test]
fn test_token_generation_with_rand_0_10() {
    let token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    assert_eq!(token.len(), 6);
    assert!(token.chars().all(|c: char| c.is_ascii_alphanumeric()));
}

#[test]
fn test_token_generation_thread_rng() {
    let token: String = rand::rngs::ThreadRng::default()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    assert_eq!(token.len(), 6);
    assert!(token.chars().all(|c: char| c.is_ascii_alphanumeric()));
}

#[test]
fn test_multiple_tokens_are_different() {
    let token1: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let token2: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    assert_ne!(token1, token2);
}
