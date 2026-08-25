use proptest::{
    prelude::prop,
    prop_oneof,
    strategy::{Just, Strategy},
    test_runner::Config,
};
use simply_db::DatabaseContext;
use storage::db::Database;

fn fuzz_random_type() -> impl Strategy<Value = String> {
    let random = proptest::collection::vec(prop::char::any(), 1..10)
        .prop_map(|chars| chars.iter().collect::<String>());
    prop_oneof![
        Just("FLOAT".to_string()),
        Just("INT".to_string()),
        Just("BOOLEAN".to_string()),
        Just("TEXT".to_string()),
        random
    ]
}
fn fuzz_field_modifiers() -> impl Strategy<Value = String> {
    let random = proptest::collection::vec(prop::char::any(), 1..15)
        .prop_map(|chars| chars.iter().collect::<String>());
    proptest::collection::vec(
        prop_oneof![
            Just("NOT NULL".to_string()),
            Just("PRIMARY KEY".to_string()),
            Just("AUTOINCREMENT".to_string()),
            Just("UNIQUE".to_string()),
            random
        ],
        0..5,
    )
    .prop_map(|mods| mods.join(" "))
}
fn fuzz_random_ident() -> impl Strategy<Value = String> {
    let random = proptest::collection::vec(prop::char::any(), 1..15)
        .prop_map(|chars| chars.iter().collect::<String>());
    prop_oneof![random, "[a-zA-Z_]{1,15}"]
}

fn single_field_strategy() -> impl Strategy<Value = String> {
    (
        fuzz_random_ident(),
        fuzz_random_type(),
        fuzz_field_modifiers(),
    )
        .prop_map(|(name, ty, modifiers)| {
            if modifiers.is_empty() {
                format!("{} {}", name, ty)
            } else {
                format!("{} {} {}", name, ty, modifiers)
            }
        })
}

fn fuzz_multiple_fields_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec(single_field_strategy(), 1..6).prop_map(|fields| fields.join(", "))
}

proptest::proptest! {
    #![proptest_config(Config::with_cases(10000))]
    #[test]
    fn test_fuzzed_create_sql(table_name in fuzz_random_ident(),fields in fuzz_multiple_fields_strategy()) {
        let db = Database::new();
        let context = DatabaseContext::new(db);
        let query = format!("CREATE TABLE {} ({})",table_name,fields);
        let res = context.execute(&query);
        match res{
            Ok(res) => if res.iter().all(|x| x.is_ok())
            {
                println!("Success with: {}",query);
            },
            _=>()
        }
    }
}
