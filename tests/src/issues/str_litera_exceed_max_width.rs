mod a {
    #[test]
    fn test_combined() {
        let args = vec!("prog".to_string(),
                     "free1".to_string(),
                     "-s".to_string(),
                     "20".to_string(),
                     "free2".to_string(),
                     "--flag".to_string(),
                     "--long=30".to_string(),
                     "-f".to_string(),
                     "-m".to_string(),
                     "40".to_string(),
                     "-m".to_string(),
                     "50".to_string(),
                     "-n".to_string(),
                     "-A B".to_string(),
                     "-n".to_string(),
                     "-60 70".to_string());
        match Options::new().optopt("s", "something", "something", "SOMETHING").optflag("", "flag",
                "a flag").reqopt("", "long", "hi", "LONG").optflag("f", "", "another flag")
                .optmulti("m", "", "mmmmmm", "YUM").optmulti("n", "", "nothing", "NOTHING").optopt(
                "", "notpresent", "nothing to see here", "NOPE").parse(&args) {}
    }
}
