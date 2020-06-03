use super::*;
use egg::{rewrite, Applier, Pattern, Rewrite, SearchMatches, Searcher, Subst, Var};
use log::{debug, info, trace};

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
        rewrite!(
            "strand-cell-associativity-0";
            "(strand-cell (strand-cell (domain ?d0) ?rest) (domain ?d1))" =>
                "(strand-cell (domain ?d0) (strand-cell ?rest (domain ?d1)))"),
        rewrite!(
            "strand-cell-associativity-1";
            "(strand-cell (domain ?d0) (strand-cell ?rest (domain ?d1)))" =>
                "(strand-cell (strand-cell (domain ?d0) ?rest) (domain ?d1))"),
    ]
}

pub fn nil_commutativity() -> Vec<Rewrite<Language, Meta>> {
    vec![
        rewrite!(
            "nil-commutativity-0";
            "(strand-cell nil ?arg)" => "(strand-cell ?arg nil)"),
        rewrite!(
            "nil-commutativity-1";
            "(strand-cell ?arg nil)" => "(strand-cell nil ?arg)"),
    ]
}

pub fn double_strand_cell_associativity() -> Vec<Rewrite<Language, Meta>> {
    vec![
        rewrite!(
            "double-strand-cell-associativity-0";
            "(double-strand-cell
              (double-strand-cell
               (strand-cell ?a ?b)
               (strand-cell ?d ?e)
               ?rest)
              (strand-cell ?g ?h)
              (strand-cell ?j ?k)
             )" =>
                "(double-strand-cell
                  (strand-cell ?a ?b)
                  (strand-cell ?d ?e)
                  (double-strand-cell
                   ?rest
                   (strand-cell ?g ?h)
                   (strand-cell ?j ?k)
                  )
                 )"),
        rewrite!(
            "double-strand-cell-associativity-1";
            "(double-strand-cell
              (strand-cell ?a ?b)
              (strand-cell ?d ?e)
              (double-strand-cell
               ?rest
               (strand-cell ?g ?h)
               (strand-cell ?j ?k)
              )
             )" =>
                "(double-strand-cell
                  (double-strand-cell
                   (strand-cell ?a ?b)
                   (strand-cell ?d ?e)
                   ?rest)
                  (strand-cell ?g ?h)
                  (strand-cell ?j ?k)
                 )"),
    ]
}

pub fn double_strand_cell_nil_commutativity() -> Vec<Rewrite<Language, Meta>> {
    vec![
        rewrite!(
            "TODO";
            "(double-strand-cell nil ?arg0 ?arg1)" => "(double-strand-cell ?arg0 ?arg1 nil)"),
        rewrite!(
            "TODO";
            "(double-strand-cell ?arg0 ?arg1 nil)" => "(double-strand-cell nil ?arg0 ?arg1)"),
    ]
}

pub fn simplify_double_complement() -> Rewrite<Language, Meta> {
    rewrite!(
        "simplify-double-complement";
        "(complement (complement ?a))" =>
            "?a")
}

/// Binds toeholds, and then binds everything after the toehold that can be
/// bound.
pub fn toehold_bind() -> Rewrite<Language, Meta> {
    const A: &'static str = "?A";
    const B: &'static str = "?B";
    let a_var: Var = A.parse().unwrap();
    let b_var: Var = B.parse().unwrap();
    struct ToeholdSearcher {
        a_var: Var,
        b_var: Var,
    };
    impl ToeholdSearcher {
        fn get_complement_match(
            &self,
            domain_id: DomainIdValue,
            egraph: &EGraph<Language, Meta>,
        ) -> Option<Id> {
            let complement_pattern: Pattern<Language> = format!(
                "(strand-cell (domain (toehold-domain (complement (domain-id {})))) nil)",
                domain_id
            )
            .parse()
            .unwrap();
            let found: Vec<SearchMatches> = complement_pattern.search(egraph);
            match found.len() {
                0 => None,
                1 => Some(found[0].eclass),
                _ => panic!("How did we find more than one eclass for this pattern?"),
            }
        }
    }
    impl Searcher<Language, Meta> for ToeholdSearcher {
        fn search_eclass(
            &self,
            egraph: &EGraph<Language, Meta>,
            eclass: Id,
        ) -> Option<SearchMatches> {
            // TODO(gus) should find everything that
            // "(strand-cell nil (domain (toehold-domain (domain-id ?domain-id))))"
            // would find.
            let pattern: Pattern<Language> =
                "(strand-cell (domain (toehold-domain (domain-id ?domain-id))) nil)"
                    .parse()
                    .unwrap();

            match pattern.search_eclass(egraph, eclass) {
                None => None,
                Some(search_matches) => {
                    let domain_id: &DomainId = match egraph
                        [search_matches.substs[0][&"?domain-id".parse().unwrap()]]
                        .metadata
                        .value
                        .as_ref()
                        .unwrap()
                    {
                        Value::DomainIdValue(v) => v,
                        _ => panic!(),
                    };
                    let domain_id_value = match domain_id {
                        DomainId::DomainId(v) => v,
                        // We should have filtered this out, given the
                        // pattern we're searching for.
                        DomainId::Complement(_) => panic!(),
                    };

                    for subst in search_matches.substs[1..].iter() {
                        assert_eq!(
                            match egraph[subst[&"?domain-id".parse().unwrap()]]
                                .metadata
                                .value
                                .as_ref()
                                .unwrap()
                            {
                                Value::DomainIdValue(v) => match v {
                                    DomainId::DomainId(v) => v,
                                    DomainId::Complement(_) => panic!(),
                                },
                                _ => panic!(),
                            },
                            domain_id_value
                        );
                    }

                    match self.get_complement_match(*domain_id_value, egraph) {
                        None => None,
                        Some(id) => Some(SearchMatches {
                            eclass: eclass,
                            substs: {
                                let mut new_subst = Subst::default();
                                new_subst.insert(self.a_var.clone(), eclass);
                                new_subst.insert(self.b_var.clone(), id);
                                vec![new_subst]
                            },
                        }),
                    }
                }
            }
        }
    }

    struct ToeholdApplier {
        a_var: Var,
        b_var: Var,
    };
    impl Applier<Language, Meta> for ToeholdApplier {
        fn apply_one(
            &self,
            egraph: &mut EGraph<Language, Meta>,
            _matched_id: Id,
            subst: &Subst,
        ) -> Vec<Id> {
            let strand_cell: Id = subst[&self.a_var];
            let strand_complement_cell: Id = subst[&self.b_var];
            let nil_eclass_id: Id = egraph.add(ENode::leaf(Language::Nil));
            egraph.add(ENode::new(
                Language::DoubleStrandCell,
                vec![strand_cell, strand_complement_cell, nil_eclass_id],
            ));
            vec![]
        }
    }

    rewrite!("toehold-bind";
             {
                 ToeholdSearcher{
                     a_var: a_var.clone(),
                     b_var: b_var.clone(),
                 }
             } => {
                 ToeholdApplier{
                     a_var: a_var.clone(),
                     b_var:b_var.clone(),
                 }
             }
    )
}

/// Rewrite which binds complementary domains, if their adjacent domains are
/// already bound.
pub fn bind() -> Rewrite<Language, Meta> {
    struct BindSearcher {
        previous_double_strand_cell: Var,
        next_bottom_strand_cell: Var,
        next_top_strand_cell: Var,
    };
    impl Searcher<Language, Meta> for BindSearcher {
        fn search_eclass(
            &self,
            egraph: &EGraph<Language, Meta>,
            eclass: Id,
        ) -> Option<SearchMatches> {
            let search_pattern: Pattern<Language> = "(double-strand-cell
                                                      ?a ?b ?c)"
                .parse()
                .unwrap();
            let matches: SearchMatches = match search_pattern.search_eclass(egraph, eclass) {
                None => return None,
                Some(m) => m,
            };

            // If there are two, one must be nil.
            let (this_bottom_cell_id, this_top_cell_id): (Id, Id) = match matches.substs.as_slice()
            {
                [subst] | [subst, _] => {
                    match egraph[subst[&"?a".parse().unwrap()]]
                        .metadata
                        .value
                        .as_ref()
                        .unwrap()
                    {
                        Value::StrandCellValue(_) => {
                            (subst[&"?a".parse().unwrap()], subst[&"?b".parse().unwrap()])
                        }
                        _ => (subst[&"?b".parse().unwrap()], subst[&"?c".parse().unwrap()]),
                    }
                }
                _ => panic!(),
            };

            let next_bottom_pattern: Pattern<Language> =
                "(strand-cell ?this-bottom-cell (domain ?domain))"
                    .parse()
                    .unwrap();
            let next_top_pattern: Pattern<Language> =
                "(strand-cell (domain ?domain) ?this-top-cell)"
                    .parse()
                    .unwrap();

            use itertools::Itertools;
            let substs_out: Vec<Subst> = next_bottom_pattern
                .search(egraph)
                .iter()
                .cartesian_product(&next_top_pattern.search(egraph))
                .filter_map(
                    |(bottom_matches, top_matches): (&SearchMatches, &SearchMatches)| {
                        // This is only true because ?domain can only have one value
                        // (will point to a long-domain or a toehold-domain) and
                        // because ?this-bottom-cell/?this-top-cell can likewise
                        // only have one value each.
                        let bottom_subst: &Subst = match bottom_matches.substs.as_slice() {
                            [subst] => subst,
                            _ => panic!(),
                        };
                        let top_subst: &Subst = match top_matches.substs.as_slice() {
                            [subst] => subst,
                            _ => panic!(),
                        };

                        // First, check that the bottom cell is the same one as
                        // the one in our double cell.
                        if bottom_subst[&"?this-bottom-cell".parse().unwrap()]
                            != this_bottom_cell_id
                        {
                            return None;
                        }

                        // Then, check that the top cell is also the one in our
                        // double cell.
                        if top_subst[&"?this-top-cell".parse().unwrap()] != this_top_cell_id {
                            return None;
                        }

                        // Finally, check that the domains are complementary.
                        match (
                            egraph[bottom_subst[&"?domain".parse().unwrap()]]
                                .metadata
                                .value
                                .as_ref()
                                .unwrap(),
                            egraph[top_subst[&"?domain".parse().unwrap()]]
                                .metadata
                                .value
                                .as_ref()
                                .unwrap(),
                        ) {
                            (
                                Value::DomainValue(Domain::Long(DomainId::DomainId(d))),
                                Value::DomainValue(Domain::Long(DomainId::Complement(d_box))),
                            )
                            | (
                                Value::DomainValue(Domain::Toehold(DomainId::DomainId(d))),
                                Value::DomainValue(Domain::Toehold(DomainId::Complement(d_box))),
                            )
                            | (
                                Value::DomainValue(Domain::Long(DomainId::Complement(d_box))),
                                Value::DomainValue(Domain::Long(DomainId::DomainId(d))),
                            )
                            | (
                                Value::DomainValue(Domain::Toehold(DomainId::Complement(d_box))),
                                Value::DomainValue(Domain::Toehold(DomainId::DomainId(d))),
                            ) => {
                                // I think there shouldn't be multiple nested complements here.
                                let d2: DomainIdValue = match **d_box {
                                    DomainId::DomainId(v) => v,
                                    _ => panic!(),
                                };
                                if *d != d2 {
                                    return None;
                                }
                            }
                            (Value::DomainValue(_), Value::DomainValue(_)) => return None,
                            _ => panic!(),
                        }

                        // If we've made it this far, these things can be stuck
                        // into a new double cell!

                        let mut subst_out = Subst::default();

                        subst_out
                            .insert(self.next_bottom_strand_cell.clone(), bottom_matches.eclass);
                        subst_out.insert(self.next_top_strand_cell.clone(), top_matches.eclass);
                        subst_out.insert(self.previous_double_strand_cell.clone(), eclass);

                        Some(subst_out)
                    },
                )
                .collect();

            Some(SearchMatches {
                eclass: eclass,
                substs: substs_out,
            })
        }
    }

    struct BindApplier {
        previous_double_strand_cell: Var,
        next_bottom_strand_cell: Var,
        next_top_strand_cell: Var,
    };
    impl Applier<Language, Meta> for BindApplier {
        fn apply_one(
            &self,
            egraph: &mut EGraph<Language, Meta>,
            _matched_id: Id,
            subst: &Subst,
        ) -> Vec<Id> {
            egraph.add(ENode::new(
                Language::DoubleStrandCell,
                vec![
                    subst[&self.previous_double_strand_cell],
                    subst[&self.next_bottom_strand_cell],
                    subst[&self.next_top_strand_cell],
                ],
            ));

            vec![]
        }
    }

    rewrite!("bind";
             { BindSearcher {
                 previous_double_strand_cell: "?previous-double-strand-cell".parse().unwrap(),
                 next_bottom_strand_cell: "?next-bottom-strand-cell".parse().unwrap(),
                 next_top_strand_cell: "?next-top-strand-cell".parse().unwrap(),
             } } =>
             { BindApplier {
                 previous_double_strand_cell: "?previous-double-strand-cell".parse().unwrap(),
                 next_bottom_strand_cell: "?next-bottom-strand-cell".parse().unwrap(),
                 next_top_strand_cell: "?next-top-strand-cell".parse().unwrap(),
             } })
}

pub fn run(egraph: &mut EGraph<Language, Meta>, rules: &[Rewrite<Language, Meta>]) {
    let mut egraph_size = egraph.total_size();
    loop {
        run_one(egraph, rules);
        if egraph_size == egraph.total_size() {
            break;
        }

        egraph_size = egraph.total_size();
    }
}

fn run_one(egraph: &mut EGraph<Language, Meta>, rules: &[Rewrite<Language, Meta>]) {
    trace!("EGraph {:?}", egraph.dump());

    let mut matches = Vec::new();
    for rule in rules {
        let ms = rule.search(&egraph);
        matches.push(ms);
    }

    for (rw, ms) in rules.iter().zip(matches) {
        let total_matches: usize = ms.iter().map(|m| m.substs.len()).sum();
        if total_matches == 0 {
            continue;
        }

        debug!("Applying {} {} times", rw.name(), total_matches);

        rw.apply(egraph, &ms);
    }

    egraph.rebuild();

    info!(
        "size: n={}, e={}",
        egraph.total_size(),
        egraph.number_of_classes()
    );
}

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

    #[test]
    fn toehold_bind_and_bind() {
        test_logger::ensure_env_logger_initialized();

        let mut egraph = EGraph::<Language, Meta>::default();
        add_strand_to_egraph(
            &mut egraph,
            &vec![
                Domain::Long(DomainId::DomainId(5)),
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::DomainId(3)),
            ],
        );

        add_strand_to_egraph(
            &mut egraph,
            &vec![
                Domain::Long(DomainId::DomainId(4)),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(1)))),
                Domain::Toehold(DomainId::Complement(Box::new(DomainId::DomainId(0)))),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(5)))),
            ],
        );

        // Rewrite strands to all their equivalent forms
        let mut rws = simplify_strand_cell();
        rws.push(toehold_bind());
        rws.push(bind());
        rws.extend(strand_cell_associativity());
        rws.extend(nil_commutativity());
        rws.extend(double_strand_cell_associativity());
        rws.extend(double_strand_cell_nil_commutativity());
        let runner = Runner::new().with_egraph(egraph).run(&rws);

        runner
            .egraph
            .dot()
            .to_svg("toehold-bind-and-bind.svg")
            .unwrap();

        assert_eq!(
            "(double-strand-cell
              (strand-cell (domain (toehold-domain (domain-id 0))) nil)
              (strand-cell (domain (toehold-domain (complement (domain-id 0)))) nil)
              nil)"
                .parse::<Pattern<Language>>()
                .unwrap()
                .search(&runner.egraph)
                .len(),
            1
        );
        assert_eq!(
            "(double-strand-cell
              (double-strand-cell
               (strand-cell ?a ?b)
               (strand-cell ?c ?d)
               ?rest)
              (strand-cell ?e ?f)
              (strand-cell ?g ?h)
             )"
            .parse::<Pattern<Language>>()
            .unwrap()
            .search(&runner.egraph)
            .len(),
            3
        );
    }
}
