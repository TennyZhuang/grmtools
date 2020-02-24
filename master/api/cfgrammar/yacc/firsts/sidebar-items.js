initSidebarItems({"struct":[["YaccFirsts","`Firsts` stores all the first sets for a given grammar. For example, given this code and grammar: `text   let grm = YaccGrammar::new(YaccKind::Original(YaccOriginalActionKind::GenericParseTree), \"     S: A 'b';     A: 'a'      | ;\").unwrap();   let firsts = Firsts::new(&grm); ` then the following assertions (and only the following assertions) about the firsts set are correct: `text   assert!(firsts.is_set(grm.rule_idx(\"S\").unwrap(), grm.token_idx(\"a\").unwrap()));   assert!(firsts.is_set(grm.rule_idx(\"S\").unwrap(), grm.token_idx(\"b\").unwrap()));   assert!(firsts.is_set(grm.rule_idx(\"A\").unwrap(), grm.token_idx(\"a\").unwrap()));   assert!(firsts.is_epsilon_set(grm.rule_idx(\"A\").unwrap())); `"]]});