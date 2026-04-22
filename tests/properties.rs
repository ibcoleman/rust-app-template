use proptest::prelude::*;
use rust_app_template::adapters::StaticGreeter;
use rust_app_template::domain::MAX_GREET_NAME_LEN;
use rust_app_template::ports::{GreetError, GreetingPort};

proptest! {
    /// Property 1: Valid names produce a response containing the name verbatim and ending with "!"
    /// For any `s: String` of length `1..=MAX_GREET_NAME_LEN` (ASCII alphanumeric + spaces),
    /// `StaticGreeter::greet(Some(&s))` returns `Ok(msg)` where `msg.contains(&s)` and `msg.ends_with("!")`.
    #[test]
    fn greet_contains_name(s in "[a-zA-Z ]{1,64}") {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(StaticGreeter::new().greet(Some(&s)));

        // Future-proof: guards against non-ASCII regex flags.
        prop_assume!(s.len() <= MAX_GREET_NAME_LEN);

        let msg = result.unwrap();
        prop_assert!(msg.contains(&s), "Expected '{}' to contain '{}'", msg, s);
        prop_assert!(msg.ends_with('!'), "Expected '{}' to end with '!'", msg);
    }

    /// Property 2: Overlong names always produce `InvalidName`
    /// For any `s: String` with `s.len() > MAX_GREET_NAME_LEN`,
    /// `greet(Some(&s))` returns `Err(GreetError::InvalidName(_))`.
    #[test]
    fn overlong_is_invalid(
        s in proptest::collection::vec(any::<char>(), (MAX_GREET_NAME_LEN + 1)..256)
            .prop_map(|chars| chars.into_iter().collect::<String>())
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let err = runtime
            .block_on(StaticGreeter::new().greet(Some(&s)))
            .unwrap_err();

        prop_assert!(
            matches!(err, GreetError::InvalidName(_)),
            "Expected InvalidName, got {:?}",
            err
        );
    }
}
