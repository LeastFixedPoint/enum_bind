use enum_bind::Bind;

#[test]
fn environments() {
    #[derive(Bind, Debug, PartialEq)]
    #[query(fn by_data_realm(data_realm: &str) -> Vec<Self>, return = Vec)]
    #[query(fn get_all() -> Vec<Self>, return = Vec)]
    enum Environment {
        #[bind(data_realm = "prod", push_stage = "prod")] Prod,
        #[bind(data_realm = "prod", push_stage = "canary")] Canary,
        #[bind(data_realm = "prod", push_stage = "staging")] StagingWithProdData,
        #[bind(data_realm = "nonprod", push_stage = "staging")] StagingWithTestData,
        #[bind(data_realm = "nonprod", push_stage = "autopush")] Autopush,
        #[bind(data_realm = "test")] IntegrationTests,
        #[bind(data_realm = "local")] Local,
    }
    use Environment::*;

    assert_eq!(Environment::by_data_realm("prod"), vec![Prod, Canary, StagingWithProdData]);
    assert_eq!(Environment::by_data_realm("nonprod"), vec![StagingWithTestData, Autopush]);

    assert_eq!(Environment::get_all(), vec![Prod, Canary, StagingWithProdData, StagingWithTestData, Autopush, IntegrationTests, Local]);
}
