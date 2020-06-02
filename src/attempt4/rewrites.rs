use super::*;
use egg::{rewrite, Rewrite};

pub fn simplify_strand_cell() -> Vec<Rewrite<Language, Meta>> {
    vec![
        rewrite!(
        "remove-empty-strand-cell-left";
        "(strand-cell (strand-cell) ?rest)" =>
            "(strand-cell ?rest)"),
        rewrite!(
        "remove-empty-strand-cell-right";
        "(strand-cell ?rest (strand-cell))" =>
            "(strand-cell ?rest)"),
        rewrite!(
        "simplify-double-strand-cell-zero-arguments";
        "(strand-cell (strand-cell))" =>
            "(strand-cell)"),
        rewrite!(
        "simplify-double-strand-cell-one-argument";
        "(strand-cell (strand-cell ?arg))" =>
            "(strand-cell ?arg)"),
        rewrite!(
        "simplify-double-strand-cell-two-arguments";
        "(strand-cell (strand-cell ?arg0 ?arg1))" =>
            "(strand-cell ?arg0 ?arg1)"),
        // These are actually invalid, given the rule that there must be zero or
        // one domains per cell.
        // rewrite!(
        //     "TODO";
        //     "(strand-cell (strand-cell ?arg0) ?arg1)" =>
        //         "(strand-cell ?arg0 ?arg1)"),
        // rewrite!(
        //     "TODO";
        //     "(strand-cell ?arg0 (strand-cell ?arg1))" =>
        //         "(strand-cell ?arg0 ?arg1)"),
    ]
}

pub fn strand_cell_associativity() -> Vec<Rewrite<Language, Meta>> {
    vec![
        // These are no longer correct, as they might break the invariant that
        // there must be zero or one domains per strand cell.
        // rewrite!(
        //     "TODO";
        //     "(strand-cell (strand-cell ?arg0 ?arg1) ?arg2)" =>
        //         "(strand-cell ?arg0 (strand-cell ?arg1 ?arg2))"),
        // rewrite!(
        //     "TODO";
        //     "(strand-cell ?arg0 (strand-cell ?arg1 ?arg2))" =>
        //         "(strand-cell (strand-cell ?arg0 ?arg1) ?arg2)"),
        rewrite!(
            "TODO";
            "(strand-cell (strand-cell (domain ?d0) ?rest) (domain ?d1))" =>
                "(strand-cell (domain ?d0) (strand-cell ?rest (domain ?d1)))"),
        rewrite!(
            "TODO";
            "(strand-cell (domain ?d0) (strand-cell ?rest (domain ?d1)))" =>
                "(strand-cell (strand-cell (domain ?d0) ?rest) (domain ?d1))"),
    ]
}

pub fn nil_commutativity() -> Vec<Rewrite<Language, Meta>> {
    vec![
        rewrite!(
            "TODO";
            "(strand-cell nil ?arg)" => "(strand-cell ?arg nil)"),
        rewrite!(
            "TODO";
            "(strand-cell ?arg nil)" => "(strand-cell nil ?arg)"),
    ]
}

pub fn rewrite() -> Rewrite<Language, Meta> {
    rewrite!(
        "rewrite";
        "(strand-cell (strand-cell ?arg0 ?arg1))" =>
            "(strand-cell ?arg0 ?arg1)")
}

pub fn simplify_double_complement() -> Rewrite<Language, Meta> {
    rewrite!(
        "simplify-double-complement";
        "(complement (complement ?a))" =>
            "?a")
}

// /// Binds toeholds, and then binds everything after the toehold that can be
// /// bound.
// pub fn toehold_bind(top_or_bottom: TopOrBottom) -> Rewrite<Language, Meta> {
//     const A: &'static str = "?A";
//     const B: &'static str = "?B";
//     let a_var: Var = A.parse().unwrap();
//     let b_var: Var = B.parse().unwrap();
//     struct ToeholdSearcher {
//         top_or_bottom: TopOrBottom,
//         /// For a given toehold domain we find, we're going to search for
//         /// its complement. We memoize the matches here.
//         _memoized_complement_matches: HashMap<DomainIdValue, Vec<Id>>,
//         a_var: Var,
//         b_var: Var,
//     };
//     impl ToeholdSearcher {
//         fn get_complement_matches(
//             &self,
//             domain_id: DomainIdValue,
//             egraph: &EGraph<Language, Meta>,
//         ) -> Vec<Id> {
//             // TODO(gus) I can't memoize, because search() and
//             // search_eclass() give non-mutable references to self :(
//             // if !self.memoized_complement_matches.contains_key(&domain_id) {
//             //     let complement_pattern: Pattern<Language> = match self.top_or_bottom {
//             //         TopOrBottom::Bottom => format!(
//             //             "(top-strand-cell (toehold-domain (complement {})) ?rest)",
//             //             domain_id
//             //         )
//             //         .parse()
//             //         .unwrap(),
//             //         TopOrBottom::Top => format!(
//             //             "(bottom-strand-cell (toehold-domain (complement {})) ?rest)",
//             //             domain_id
//             //         )
//             //         .parse()
//             //         .unwrap(),
//             //     };
//             //     self.memoized_complement_matches.insert(
//             //         domain_id,
//             //         complement_pattern
//             //             .search(egraph)
//             //             .iter()
//             //             .map(|search_matches: &SearchMatches| search_matches.eclass)
//             //             .collect(),
//             //     );
//             // }
//             // self.memoized_complement_matches
//             //     .get(&domain_id)
//             //     .unwrap()
//             //     .clone()
//             let complement_pattern: Pattern<Language> = match self.top_or_bottom {
//                 TopOrBottom::Bottom => format!(
//                     "(top-strand-cell (toehold-domain (complement (domain-id {}))) ?rest)",
//                     domain_id
//                 )
//                 .parse()
//                 .unwrap(),
//                 TopOrBottom::Top => format!(
//                     "(bottom-strand-cell (toehold-domain (complement (domain-id {}))) ?rest)",
//                     domain_id
//                 )
//                 .parse()
//                 .unwrap(),
//             };
//             complement_pattern
//                 .search(egraph)
//                 .iter()
//                 .map(|search_matches: &SearchMatches| search_matches.eclass)
//                 .collect()
//         }
//     }
//     impl Searcher<Language, Meta> for ToeholdSearcher {
//         fn search_eclass(
//             &self,
//             egraph: &EGraph<Language, Meta>,
//             eclass: Id,
//         ) -> Option<SearchMatches> {
//             let pattern: Pattern<Language> = match self.top_or_bottom {
//                 TopOrBottom::Bottom => format!(
//                     "(bottom-strand-cell (toehold-domain (domain-id {})) ?rest)",
//                     A
//                 )
//                 .parse()
//                 .unwrap(),

//                 TopOrBottom::Top => {
//                     format!("(top-strand-cell (toehold-domain (domain-id {})) ?rest)", A)
//                         .parse()
//                         .unwrap()
//                 }
//             };

//             match pattern.search_eclass(egraph, eclass) {
//                 None => None,
//                 Some(search_matches) => {
//                     let domain_id: &DomainId = egraph[search_matches.substs[0][&self.a_var]]
//                         .metadata
//                         .domain_id
//                         .as_ref()
//                         .unwrap();
//                     let domain_id_value = match domain_id {
//                         DomainId::DomainId(v) => v,
//                         // We should have filtered this out, given the
//                         // pattern we're searching for.
//                         DomainId::Complement(_) => panic!(),
//                     };

//                     let complement_matches: Vec<Id> =
//                         self.get_complement_matches(*domain_id_value, egraph);
//                     if complement_matches.len() == 0 {
//                         return None;
//                     }

//                     Some(SearchMatches {
//                         eclass: search_matches.eclass,
//                         substs: complement_matches
//                             .iter()
//                             .map(|id: &Id| {
//                                 let mut new_subst = Subst::default();
//                                 new_subst.insert(self.a_var.clone(), search_matches.eclass);
//                                 new_subst.insert(self.b_var.clone(), *id);
//                                 new_subst
//                             })
//                             .collect(),
//                     })
//                 }
//             }
//         }
//     }

//     struct ToeholdApplier {
//         a_var: Var,
//         b_var: Var,
//         top_or_bottom: TopOrBottom,
//     };
//     impl Applier<Language, Meta> for ToeholdApplier {
//         fn apply_one(
//             &self,
//             egraph: &mut EGraph<Language, Meta>,
//             _matched_id: Id,
//             subst: &Subst,
//         ) -> Vec<Id> {
//             let single_strand_cell: Id = subst[&self.a_var];
//             let single_strand_complement_cell: Id = subst[&self.b_var];
//             let nil_eclass_id: Id = egraph.add(ENode::leaf(Language::Nil));
//             egraph.add(ENode::new(
//                 match self.top_or_bottom {
//                     TopOrBottom::Bottom => Language::BottomDoubleStrandCell,
//                     TopOrBottom::Top => Language::TopDoubleStrandCell,
//                 },
//                 vec![
//                     single_strand_cell,
//                     single_strand_complement_cell,
//                     nil_eclass_id,
//                 ],
//             ));
//             vec![]
//         }
//     }

//     rewrite!("toehold-bind";
//              {
//                  ToeholdSearcher{
//                      a_var: a_var.clone(),
//                      b_var: b_var.clone(),
//                      _memoized_complement_matches: HashMap::default(),
//                      top_or_bottom: top_or_bottom
//                  }
//              } => {
//                  ToeholdApplier{
//                      a_var: a_var.clone(),
//                      b_var:b_var.clone(),
//                      top_or_bottom:top_or_bottom
//                  }
//              }
//     )
// }

// /// Rewrite which binds complementary domains, if their adjacent domains are
// /// already bound.
// pub fn bind(top_or_bottom: TopOrBottom) -> Rewrite<Language, Meta> {
//     let search_pattern: Pattern<Language> = match top_or_bottom {
//         TopOrBottom::Bottom => {
//             "(bottom-double-strand-cell
//                                      (bottom-strand-cell
//                                       ?unused0
//                                       (bottom-strand-cell
//                                        (long-domain (domain-id ?a))
//                                        ?strand0rest))
//                                      (top-strand-cell
//                                       ?unused2
//                                       (top-strand-cell
//                                        (long-domain (complement (domain-id ?a)))
//                                        ?strand1rest))
//                                      ?unused4)"
//         }
//         TopOrBottom::Top => {
//             "(top-double-strand-cell
//                                   (top-strand-cell
//                                    ?unused0
//                                    (top-strand-cell
//                                     (long-domain (domain-id ?a))
//                                     ?strand0rest))
//                                   (bottom-strand-cell
//                                    ?unused2
//                                    (bottom-strand-cell
//                                     (long-domain (complement (domain-id ?a)))
//                                     ?strand1rest))
//                                   ?unused4)"
//         }
//     }
//     .parse()
//     .unwrap();

//     struct BindApplier {
//         top_or_bottom: TopOrBottom,
//         domain: Var,
//         strand_0_rest: Var,
//         strand_1_rest: Var,
//     }
//     impl Applier<Language, Meta> for BindApplier {
//         fn apply_one(
//             &self,
//             egraph: &mut EGraph<Language, Meta>,
//             matched_id: Id,
//             subst: &Subst,
//         ) -> Vec<Id> {
//             let domain_id_value: DomainIdValue = match egraph[subst[&self.domain]]
//                 .metadata
//                 .domain_id
//                 .as_ref()
//                 .unwrap()
//             {
//                 DomainId::DomainId(v) => *v,
//                 _ => panic!(),
//             };

//             let domain_id_value_eclass_id: Id =
//                 egraph.add(ENode::leaf(Language::DomainIdValue(domain_id_value)));
//             let domain_id_eclass_id: Id = egraph.add(ENode::new(
//                 Language::DomainId,
//                 vec![domain_id_value_eclass_id],
//             ));
//             let long_domain_eclass_id: Id =
//                 egraph.add(ENode::new(Language::LongDomain, vec![domain_id_eclass_id]));
//             let complement_eclass_id: Id =
//                 egraph.add(ENode::new(Language::Complement, vec![domain_id_eclass_id]));
//             let complement_long_domain_eclass_id: Id =
//                 egraph.add(ENode::new(Language::LongDomain, vec![complement_eclass_id]));
//             let single_strand_0_eclass_id = egraph.add(ENode::new(
//                 match self.top_or_bottom {
//                     TopOrBottom::Top => Language::TopStrandCell,
//                     TopOrBottom::Bottom => Language::BottomStrandCell,
//                 },
//                 vec![long_domain_eclass_id, subst[&self.strand_0_rest]],
//             ));
//             let single_strand_1_eclass_id = egraph.add(ENode::new(
//                 match self.top_or_bottom {
//                     TopOrBottom::Top => Language::BottomStrandCell,
//                     TopOrBottom::Bottom => Language::TopStrandCell,
//                 },
//                 vec![complement_long_domain_eclass_id, subst[&self.strand_1_rest]],
//             ));

//             egraph.add(ENode::new(
//                 match self.top_or_bottom {
//                     TopOrBottom::Top => Language::TopDoubleStrandCell,
//                     TopOrBottom::Bottom => Language::BottomDoubleStrandCell,
//                 },
//                 vec![
//                     single_strand_0_eclass_id,
//                     single_strand_1_eclass_id,
//                     matched_id,
//                 ],
//             ));

//             vec![]
//         }
//     }

//     rewrite!("bind";
//     search_pattern =>
//     {
//         BindApplier {
//             domain: "?a".parse().unwrap(),
//             top_or_bottom: top_or_bottom,
//             strand_0_rest: "?strand0rest".parse().unwrap(),
//             strand_1_rest: "?strand1rest".parse().unwrap(),
//         }})
// }

// pub fn run(egraph: &mut EGraph<Language, Meta>, rules: &[Rewrite<Language, Meta>]) {
//     let mut egraph_size = egraph.total_size();
//     loop {
//         run_one(egraph, rules);
//         if egraph_size == egraph.total_size() {
//             break;
//         }

//         egraph_size = egraph.total_size();
//     }
// }

// fn run_one(egraph: &mut EGraph<Language, Meta>, rules: &[Rewrite<Language, Meta>]) {
//     trace!("EGraph {:?}", egraph.dump());

//     let mut matches = Vec::new();
//     for rule in rules {
//         let ms = rule.search(&egraph);
//         matches.push(ms);
//     }

//     for (rw, ms) in rules.iter().zip(matches) {
//         let total_matches: usize = ms.iter().map(|m| m.substs.len()).sum();
//         if total_matches == 0 {
//             continue;
//         }

//         debug!("Applying {} {} times", rw.name(), total_matches);

//         rw.apply(egraph, &ms);
//     }

//     egraph.rebuild();

//     info!(
//         "size: n={}, e={}",
//         egraph.total_size(),
//         egraph.number_of_classes()
//     );
// }

#[cfg(test)]
mod tests {
    use super::*;
    use egg::{EGraph, ENode, Pattern, Runner, Searcher};

    #[test]
    fn simplify_double_complement() {
        let mut egraph = EGraph::default();

        let enode_id = egraph.add(ENode::leaf(Language::DomainIdValue(0)));
        let enode_id = egraph.add(ENode::new(Language::Complement, vec![enode_id]));
        let enode_id = egraph.add(ENode::new(Language::Complement, vec![enode_id]));

        assert!(!egraph[enode_id]
            .nodes
            .iter()
            .any(|enode| { enode.op == Language::DomainIdValue(0) }));
        let runner = Runner::new()
            .with_egraph(egraph)
            .run(&[super::simplify_double_complement()]);
        assert!(runner.egraph[enode_id]
            .nodes
            .iter()
            .any(|enode| { enode.op == Language::DomainIdValue(0) }));
    }

    #[test]
    fn strand_rewrites() {
        let mut egraph = EGraph::<Language, Meta>::default();
        add_strand_to_egraph(
            &mut egraph,
            &vec![
                Domain::Long(DomainId::DomainId(5)),
                Domain::Long(DomainId::DomainId(4)),
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                Domain::Long(DomainId::DomainId(3)),
            ],
        );

        let mut rws = simplify_strand_cell();
        rws.extend(strand_cell_associativity());
        rws.extend(nil_commutativity());
        let runner = Runner::new().with_egraph(egraph).run(&rws);

        //runner.egraph.dot().to_svg("simplify-associativity-commutativity.svg").unwrap();

        assert_eq!(
            "
             (strand-cell
              (strand-cell
               (strand-cell
                (strand-cell
                 (strand-cell
                  (strand-cell
                   (strand-cell
                    nil
                    (domain (long-domain (domain-id 5)))
                   )
                   (domain (long-domain (domain-id 4)))
                  )
                  (domain (toehold-domain (domain-id 0)))
                 )
                 (domain (long-domain (domain-id 1)))
                )
                (domain (long-domain (domain-id 2)))
               )
               (domain (long-domain (complement (domain-id 2))))
              )
              (domain (long-domain (domain-id 3)))
             )
             "
            .parse::<Pattern<_>>()
            .unwrap()
            .search(&runner.egraph)
            .len(),
            1
        );

        assert_eq!(
            "
             (strand-cell (domain (long-domain (domain-id 5)))
              (strand-cell (domain (long-domain (domain-id 4)))
               (strand-cell (domain (toehold-domain (domain-id 0)))
                (strand-cell (domain (long-domain (domain-id 1)))
                 (strand-cell (domain (long-domain (domain-id 2)))
                  (strand-cell (domain (long-domain (complement (domain-id 2))))
                   (strand-cell (domain (long-domain (domain-id 3)))
                                nil)))))))
             "
            .parse::<Pattern<_>>()
            .unwrap()
            .search(&runner.egraph)
            .len(),
            1
        );

        // toehold should get isolated.
        assert_eq!(
            "(strand-cell
              (domain (toehold-domain (domain-id 0)))
              nil)
             "
            .parse::<Pattern<_>>()
            .unwrap()
            .search(&runner.egraph)
            .len(),
            1
        );
        assert_eq!(
            "(strand-cell
              nil
              (domain (toehold-domain (domain-id 0))))
             "
            .parse::<Pattern<_>>()
            .unwrap()
            .search(&runner.egraph)
            .len(),
            1
        );

        // Should be able to easily search for neighbors.
        assert!(
            "(strand-cell
              (domain (toehold-domain (domain-id 0)))
              (strand-cell (domain (long-domain (domain-id 1))) ?rest))
             "
            .parse::<Pattern<_>>()
            .unwrap()
            .search(&runner.egraph)
            .len()
                > 0
        );
    }

    // #[test]
    // fn toehold_bind_and_bind() {
    //     test_logger::ensure_env_logger_initialized();

    //     let mut egraph = EGraph::<Language, Meta>::default();
    //     add_strand_to_egraph(
    //         &mut egraph,
    //         &vec![
    //             Domain::Toehold(DomainId::DomainId(0)),
    //             Domain::Long(DomainId::DomainId(1)),
    //             Domain::Long(DomainId::DomainId(2)),
    //             Domain::Long(DomainId::DomainId(3)),
    //         ],
    //     );

    //     add_strand_to_egraph(
    //         &mut egraph,
    //         &vec![
    //             Domain::Toehold(DomainId::Complement(Box::new(DomainId::DomainId(0)))),
    //             Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(1)))),
    //             Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
    //             Domain::Long(DomainId::DomainId(4)),
    //         ],
    //     );

    //     run(
    //         &mut egraph,
    //         &[
    //             super::toehold_bind(TopOrBottom::Bottom),
    //             super::bind(TopOrBottom::Bottom),
    //         ],
    //     );

    //     assert_eq!(
    //         "(bottom-double-strand-cell ?a ?b
    //               (bottom-double-strand-cell ?c ?d
    //                (bottom-double-strand-cell ?e ?f nil)))"
    //             .parse::<Pattern<Language>>()
    //             .unwrap()
    //             .search(&egraph)
    //             .len(),
    //         1
    //     );
    // }
}
