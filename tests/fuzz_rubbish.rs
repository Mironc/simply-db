use proptest::strategy::Strategy;
use proptest::test_runner::Config;
use simply_db::DatabaseContext;
use simply_db::storage::db::Database;

fn fuzz_rubbish_token() -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::any(), 1..30)
        .prop_map(|x| x.iter().collect::<String>())
}
fn fuzz_rubbish_string() -> impl Strategy<Value = String> {
    proptest::collection::vec(fuzz_rubbish_token(), 1..30).prop_map(|x| x.join(" "))
}
proptest::proptest! {
    #![proptest_config(Config::with_cases(10000))]
    #[test]
    fn test_fuzzed_rubbish(rubbish in fuzz_rubbish_string()) {
        let db = Database::new();
        let context = DatabaseContext::new(db);
        println!("{}",rubbish);
        _ = context.execute(&rubbish);
    }
}
