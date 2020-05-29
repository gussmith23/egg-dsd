use egg::{define_language, Applier, EGraph, ENode, Id, Metadata, Var};
use std::collections::HashMap;

type DomainIdValue = u32;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainId {
    Complement(Box<DomainId>),
    DomainId(DomainIdValue),
}
type StrandId = u32;

pub enum Domain {
    Toehold(DomainId),
    Long(DomainId),
}

#[derive(Copy, Clone)]
pub enum TopOrBottom {
    Top,
    Bottom,
}

define_language! {
    pub enum Language {
        // Syntax:

        // double-strand-cell: [(bottom-double-strand-cell
        //                       <bottom-strand-cell>
        //                       <top-strand-cell>
        //                       [<bottom-double-strand-cell>|nil] )
        //                      | (top-double-strand-cell
        //                         <top-strand-cell>
        //                         <bottom-strand-cell>
        //                         [<top-double-strand-cell>|nil] )]
        BottomDoubleStrandCell = "bottom-double-strand-cell",
        TopDoubleStrandCell = "top-double-strand-cell",


        // a unique strand: (strand <strand-id> <strand-cell>)
        Strand = "strand",

        // strand-cell: [(bottom-strand-cell <domain> [<strand-cell> | nil])
        //               | (top-strand-cell <domain> [<strand-cell> | nil])]
        BottomStrandCell = "bottom-strand-cell",
        TopStrandCell = "top-strand-cell",

        // domain: [ (long-domain <domain-id>)
        //          | (toehold-domain <domain-id>)]
        LongDomain = "long-domain",
        ToeholdDomain = "toehold-domain",

        // domain-id: [ (complement <domain-id>)
        //              | (domain-id <DomainIdValue>) ]
        Complement = "complement",
        DomainId = "domain-id",


        // TODO(gus) give an alias for u32 here?
        DomainIdValue(u32),

        Nil = "nil",
        StrandId(StrandId),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta {
    /// If this eclass contains a domain id, this will hold its value.
    domain_id: Option<DomainId>,
}
impl Metadata<Language> for Meta {
    type Error = ();

    fn merge(&self, other: &Self) -> Self {
        assert_eq!(self, other);
        self.clone()
    }

    fn make(egraph: &EGraph<Language, Self>, enode: &ENode<Language>) -> Self {
        match &enode.op {
            Language::BottomDoubleStrandCell => Meta { domain_id: None },
            Language::TopDoubleStrandCell => Meta { domain_id: None },
            Language::DomainIdValue(id) => Meta {
                domain_id: Some(DomainId::DomainId(*id)),
            },
            Language::BottomStrandCell => {
                assert_eq!(enode.children.len(), 2);
                Meta {
                    domain_id: Some(
                        egraph[enode.children[0]]
                            .metadata
                            .domain_id
                            .as_ref()
                            .unwrap()
                            .clone(),
                    ),
                }
            }
            Language::TopStrandCell => {
                assert_eq!(enode.children.len(), 2);
                Meta {
                    domain_id: Some(
                        egraph[enode.children[0]]
                            .metadata
                            .domain_id
                            .as_ref()
                            .unwrap()
                            .clone(),
                    ),
                }
            }
            Language::Strand => Meta { domain_id: None },
            Language::LongDomain => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    domain_id: Some(
                        egraph[enode.children[0]]
                            .metadata
                            .domain_id
                            .as_ref()
                            .unwrap()
                            .clone(),
                    ),
                }
            }
            Language::ToeholdDomain => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    domain_id: Some(
                        egraph[enode.children[0]]
                            .metadata
                            .domain_id
                            .as_ref()
                            .unwrap()
                            .clone(),
                    ),
                }
            }
            Language::DomainId => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    domain_id: Some(
                        egraph[enode.children[0]]
                            .metadata
                            .domain_id
                            .as_ref()
                            .unwrap()
                            .clone(),
                    ),
                }
            }
            Language::Complement => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    domain_id: Some(
                        match egraph[enode.children[0]]
                            .metadata
                            .domain_id
                            .as_ref()
                            .unwrap()
                        {
                            DomainId::Complement(domain_id_box) => {
                                DomainId::DomainId(match **domain_id_box {
                                    DomainId::Complement(_) => panic!(),
                                    DomainId::DomainId(v) => v,
                                })
                            }
                            DomainId::DomainId(domain_id) => {
                                DomainId::Complement(Box::new(DomainId::DomainId(*domain_id)))
                            }
                        },
                    ),
                }
            }
            Language::Nil => Meta { domain_id: None },
            Language::StrandId(_) => Meta { domain_id: None },
        }
    }
}

pub fn add_strand_to_egraph(
    egraph: &mut EGraph<Language, Meta>,
    top_or_bottom: TopOrBottom,
    strand_id: StrandId,
    strand_values: &Vec<Domain>,
) -> Id {
    let nil_id = egraph.add(ENode::leaf(Language::Nil));

    // This fold starts from the back of the list, adding each domain and
    // wrapping it in a strand cell. We need to go back-to-front so that we know
    // which strand-cell to point to in the "next" field of each strand cell.
    let first_strand_cell_id: Id = strand_values.iter().rev().fold(
        nil_id,
        |next_strand_cell_enode_id: Id, domain: &Domain| {
            fn add_domain_id_to_egraph(
                egraph: &mut EGraph<Language, Meta>,
                domain_id: &DomainId,
            ) -> Id {
                match &domain_id {
                    &DomainId::Complement(domain_id) => {
                        let domain_id_egraph_id: Id = add_domain_id_to_egraph(egraph, domain_id);
                        egraph.add(ENode::new(Language::Complement, vec![domain_id_egraph_id]))
                    }
                    &DomainId::DomainId(id) => {
                        let domain_id_value_egraph_id: Id =
                            egraph.add(ENode::leaf(Language::DomainIdValue(*id)));
                        egraph.add(ENode::new(
                            Language::DomainId,
                            vec![domain_id_value_egraph_id],
                        ))
                    }
                }
            }
            fn add_domain_to_egraph(egraph: &mut EGraph<Language, Meta>, domain: &Domain) -> Id {
                match &domain {
                    &Domain::Toehold(id) => {
                        let domain_id_enode_id: Id = add_domain_id_to_egraph(egraph, id);
                        egraph.add(ENode::new(
                            Language::ToeholdDomain,
                            vec![domain_id_enode_id],
                        ))
                    }
                    &Domain::Long(id) => {
                        let domain_id_enode_id: Id = add_domain_id_to_egraph(egraph, id);
                        egraph.add(ENode::new(Language::LongDomain, vec![domain_id_enode_id]))
                    }
                }
            }

            // Put the domain into an egraph node, and get the id.
            let domain_enode_id: Id = add_domain_to_egraph(egraph, domain);

            egraph.add(ENode::new(
                match top_or_bottom {
                    TopOrBottom::Bottom => Language::BottomStrandCell,
                    TopOrBottom::Top => Language::TopStrandCell,
                },
                vec![domain_enode_id, next_strand_cell_enode_id],
            ))
        },
    );

    let strand_id_enode_id: Id = egraph.add(ENode::leaf(Language::StrandId(strand_id)));
    let out = egraph.add(ENode::new(
        Language::Strand,
        vec![strand_id_enode_id, first_strand_cell_id],
    ));

    egraph.rebuild();

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use egg::{EGraph, Pattern, Searcher};

    #[test]
    fn add_to_egraph() {
        let mut egraph = EGraph::<Language, Meta>::default();
        add_strand_to_egraph(
            &mut egraph,
            TopOrBottom::Bottom,
            0,
            &vec![
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                Domain::Long(DomainId::DomainId(3)),
            ],
        );

        //egraph.dot().to_svg("tmp.svg").unwrap();
    }

    #[test]
    fn search() {
        let mut egraph = EGraph::<Language, Meta>::default();
        add_strand_to_egraph(
            &mut egraph,
            TopOrBottom::Bottom,
            0,
            &vec![
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                Domain::Long(DomainId::DomainId(3)),
            ],
        );

        add_strand_to_egraph(
            &mut egraph,
            TopOrBottom::Top,
            0,
            &vec![
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::DomainId(4)),
            ],
        );

        //egraph.dot().to_svg("tmp2.svg").unwrap();

        let bottom_pattern: Pattern<Language> = "(bottom-strand-cell (toehold-domain ?a) ?rest)"
            .parse()
            .unwrap();
        let top_pattern: Pattern<Language> = "(top-strand-cell (toehold-domain ?a) ?rest)"
            .parse()
            .unwrap();

        let bottom_matches = bottom_pattern.search(&egraph);
        assert_eq!(bottom_matches.len(), 1);
        assert_eq!(bottom_matches[0].substs.len(), 1);
        let top_matches = top_pattern.search(&egraph);
        assert_eq!(top_matches.len(), 1);
        assert_eq!(top_matches[0].substs.len(), 1);
    }
}

pub mod rewrites {
    use super::*;
    use egg::{rewrite, Pattern, Rewrite, SearchMatches, Searcher, Subst};

    pub fn simplify_double_complement() -> Rewrite<Language, Meta> {
        rewrite!(
        "simplify-double-complement";
        "(complement (complement ?a))" =>
            "?a")
    }

    /// Binds toeholds, and then binds everything after the toehold that can be
    /// bound.
    pub fn toehold_bind(top_or_bottom: TopOrBottom) -> Rewrite<Language, Meta> {
        const A: &'static str = "?A";
        const B: &'static str = "?B";
        let a_var: Var = A.parse().unwrap();
        let b_var: Var = B.parse().unwrap();
        struct ToeholdSearcher {
            top_or_bottom: TopOrBottom,
            /// For a given toehold domain we find, we're going to search for
            /// its complement. We memoize the matches here.
            _memoized_complement_matches: HashMap<DomainIdValue, Vec<Id>>,
            a_var: Var,
            b_var: Var,
        };
        impl ToeholdSearcher {
            fn get_complement_matches(
                &self,
                domain_id: DomainIdValue,
                egraph: &EGraph<Language, Meta>,
            ) -> Vec<Id> {
                // TODO(gus) I can't memoize, because search() and
                // search_eclass() give non-mutable references to self :(
                // if !self.memoized_complement_matches.contains_key(&domain_id) {
                //     let complement_pattern: Pattern<Language> = match self.top_or_bottom {
                //         TopOrBottom::Bottom => format!(
                //             "(top-strand-cell (toehold-domain (complement {})) ?rest)",
                //             domain_id
                //         )
                //         .parse()
                //         .unwrap(),
                //         TopOrBottom::Top => format!(
                //             "(bottom-strand-cell (toehold-domain (complement {})) ?rest)",
                //             domain_id
                //         )
                //         .parse()
                //         .unwrap(),
                //     };
                //     self.memoized_complement_matches.insert(
                //         domain_id,
                //         complement_pattern
                //             .search(egraph)
                //             .iter()
                //             .map(|search_matches: &SearchMatches| search_matches.eclass)
                //             .collect(),
                //     );
                // }
                // self.memoized_complement_matches
                //     .get(&domain_id)
                //     .unwrap()
                //     .clone()
                let complement_pattern: Pattern<Language> = match self.top_or_bottom {
                    TopOrBottom::Bottom => format!(
                        "(top-strand-cell (toehold-domain (complement (domain-id {}))) ?rest)",
                        domain_id
                    )
                    .parse()
                    .unwrap(),
                    TopOrBottom::Top => format!(
                        "(bottom-strand-cell (toehold-domain (complement (domain-id {}))) ?rest)",
                        domain_id
                    )
                    .parse()
                    .unwrap(),
                };
                complement_pattern
                    .search(egraph)
                    .iter()
                    .map(|search_matches: &SearchMatches| search_matches.eclass)
                    .collect()
            }
        }
        impl Searcher<Language, Meta> for ToeholdSearcher {
            fn search_eclass(
                &self,
                egraph: &EGraph<Language, Meta>,
                eclass: Id,
            ) -> Option<SearchMatches> {
                let pattern: Pattern<Language> = match self.top_or_bottom {
                    TopOrBottom::Bottom => format!(
                        "(bottom-strand-cell (toehold-domain (domain-id {})) ?rest)",
                        A
                    )
                    .parse()
                    .unwrap(),

                    TopOrBottom::Top => {
                        format!("(top-strand-cell (toehold-domain (domain-id {})) ?rest)", A)
                            .parse()
                            .unwrap()
                    }
                };

                match pattern.search_eclass(egraph, eclass) {
                    None => None,
                    Some(search_matches) => {
                        let domain_id: &DomainId = egraph[search_matches.substs[0][&self.a_var]]
                            .metadata
                            .domain_id
                            .as_ref()
                            .unwrap();
                        let domain_id_value = match domain_id {
                            DomainId::DomainId(v) => v,
                            // We should have filtered this out, given the
                            // pattern we're searching for.
                            DomainId::Complement(_) => panic!(),
                        };

                        let complement_matches: Vec<Id> =
                            self.get_complement_matches(*domain_id_value, egraph);
                        if complement_matches.len() == 0 { return None }

                        Some(SearchMatches {
                            eclass: search_matches.eclass,
                            substs: complement_matches
                                .iter()
                                .map(|id: &Id| {
                                    let mut new_subst = Subst::default();
                                    new_subst.insert(self.a_var.clone(), search_matches.eclass);
                                    new_subst.insert(self.b_var.clone(), *id);
                                    new_subst
                                })
                                .collect(),
                        })
                    }
                }
            }
        }

        struct ToeholdApplier {
            a_var: Var,
            b_var: Var,
            top_or_bottom: TopOrBottom,
        };
        impl Applier<Language, Meta> for ToeholdApplier {
            fn apply_one(
                &self,
                egraph: &mut EGraph<Language, Meta>,
                _matched_id: Id,
                subst: &Subst,
            ) -> Vec<Id> {
                let single_strand_cell: Id = subst[&self.a_var];
                let single_strand_complement_cell: Id = subst[&self.b_var];
                let nil_eclass_id: Id = egraph.add(ENode::leaf(Language::Nil));
                egraph.add(ENode::new(
                    match self.top_or_bottom {
                        TopOrBottom::Bottom => Language::BottomDoubleStrandCell,
                        TopOrBottom::Top => Language::TopDoubleStrandCell,
                    },
                    vec![
                        single_strand_cell,
                        single_strand_complement_cell,
                        nil_eclass_id,
                    ],
                ));
                vec![]
            }
        }

        rewrite!("toehold-bind";
                 {
                     ToeholdSearcher{
                         a_var: a_var.clone(),
                         b_var: b_var.clone(),
                         _memoized_complement_matches: HashMap::default(),
                         top_or_bottom: top_or_bottom
                     }
                 } => {
                     ToeholdApplier{
                         a_var: a_var.clone(),
                         b_var:b_var.clone(),
                         top_or_bottom:top_or_bottom
                     }
                 }
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use egg::{EGraph, ENode, Runner};

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
        fn toehold_bind() {
            let mut egraph = EGraph::<Language, Meta>::default();
            add_strand_to_egraph(
                &mut egraph,
                TopOrBottom::Bottom,
                0,
                &vec![
                    Domain::Toehold(DomainId::DomainId(0)),
                    Domain::Long(DomainId::DomainId(1)),
                    Domain::Long(DomainId::DomainId(2)),
                    Domain::Long(DomainId::DomainId(3)),
                ],
            );

            add_strand_to_egraph(
                &mut egraph,
                TopOrBottom::Top,
                1,
                &vec![
                    Domain::Toehold(DomainId::Complement(Box::new(DomainId::DomainId(0)))),
                    Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(1)))),
                    Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                    Domain::Long(DomainId::DomainId(4)),
                ],
            );
            let runner = Runner::new()
                .with_egraph(egraph)
                .run(&[super::toehold_bind(TopOrBottom::Bottom)]);

            runner.egraph.dot().to_svg("out.svg").unwrap();
            assert!("(bottom-double-strand-cell ?a ?b ?rest)".parse::<Pattern<Language>>().unwrap().search(&runner.egraph).len() > 0);
        }
    }
}
