use egg::{define_language, EGraph, ENode, Id, Metadata};

pub mod rewrites;

type DomainIdValue = u32;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainId {
    Complement(Box<DomainId>),
    DomainId(DomainIdValue),
}
//type StrandId = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Domain {
    Toehold(DomainId),
    Long(DomainId),
}

define_language! {
    pub enum Language {
        // Syntax:

        // double-strand-cell: [ (double-strand-cell
        //                        <bottom-strand-cell>
        //                        <top-strand-cell>
        //                        [<bottom-double-strand-cell>|nil] )
        //                      | (double-strand-cell
        //                         [<bottom-double-strand-cell>|nil]
        //                         <bottom-strand-cell>
        //                         <top-strand-cell>)
        DoubleStrandCell = "double-strand-cell",

        // Note that a strand cell has zero or one domains; no more.
        // strand-cell: [| (strand-cell [ <strand-cell> | nil ] <domain>)
        //               | (strand-cell <domain> [ <strand-cell> | nil ])
        //               | (strand-cell [ <strand-cell> | nil ] [ <strand-cell> | nil ]) ]
        StrandCell = "strand-cell",

        // domain: [ (domain (long-domain <domain-id>))
        //          | (domain (toehold-domain <domain-id>)) ]
        Domain = "domain",
        LongDomain = "long-domain",
        ToeholdDomain = "toehold-domain",

        // domain-id: [ (complement <domain-id>)
        //              | (domain-id <DomainIdValue>) ]
        Complement = "complement",
        DomainId = "domain-id",

        Nil = "nil",

        // TODO(gus) give an alias for u32 here?
        DomainIdValue(u32),
    }
}

/// domain-id nodes and strand-cell nodes should never be unified!
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// The value taken on by a domain-id node.
    DomainIdValue(DomainId),
    /// The value taken on by a domain node (long-domain, toehold-domain, domain)
    DomainValue(Domain),
    /// A strand's value is a sequence of domains.
    StrandCellValue(Vec<Domain>),
    /// A double strand's value is also a sequence of domains representing the
    /// bottom strand; the top strand's values are all complementary to the
    /// bottom strand.
    DoubleStrandCellValue(Vec<Domain>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta {
    /// If this eclass contains a domain id, this will hold its value.
    value: Option<Value>,
}
impl Metadata<Language> for Meta {
    type Error = ();

    fn merge(&self, other: &Self) -> Self {
        assert_eq!(self, other);
        self.clone()
    }

    fn make(egraph: &EGraph<Language, Self>, enode: &ENode<Language>) -> Self {
        match &enode.op {
            Language::Nil => Meta { value: None },
            Language::DomainIdValue(id) => Meta {
                value: Some(Value::DomainIdValue(DomainId::DomainId(*id))),
            },
            Language::Domain => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    value: Some(
                        match egraph[enode.children[0]].metadata.value.as_ref().unwrap() {
                            Value::DomainValue(v) => Value::DomainValue(v.clone()),
                            _ => panic!(),
                        },
                    ),
                }
            }
            Language::LongDomain => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    value: Some(
                        match egraph[enode.children[0]].metadata.value.as_ref().unwrap() {
                            Value::DomainIdValue(v) => Value::DomainValue(Domain::Long(v.clone())),
                            _ => panic!(),
                        },
                    ),
                }
            }
            Language::ToeholdDomain => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    value: Some(
                        match egraph[enode.children[0]].metadata.value.as_ref().unwrap() {
                            Value::DomainIdValue(v) => {
                                Value::DomainValue(Domain::Toehold(v.clone()))
                            }
                            _ => panic!(),
                        },
                    ),
                }
            }
            Language::DomainId => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    value: Some(
                        match egraph[enode.children[0]].metadata.value.as_ref().unwrap() {
                            Value::DomainIdValue(v) => Value::DomainIdValue(v.clone()),
                            _ => panic!(),
                        },
                    ),
                }
            }
            Language::Complement => {
                assert_eq!(enode.children.len(), 1);
                Meta {
                    value: Some(
                        match egraph[enode.children[0]].metadata.value.as_ref().unwrap() {
                            Value::DomainIdValue(DomainId::Complement(domain_id_box)) => {
                                Value::DomainIdValue(DomainId::DomainId(match **domain_id_box {
                                    DomainId::Complement(_) => panic!(),
                                    DomainId::DomainId(v) => v,
                                }))
                            }
                            Value::DomainIdValue(DomainId::DomainId(domain_id)) => {
                                Value::DomainIdValue(DomainId::Complement(Box::new(
                                    DomainId::DomainId(*domain_id),
                                )))
                            }
                            _ => panic!(),
                        },
                    ),
                }
            }
            // At first I thought StrandCells should take on the value of the
            // domain they contain. I'm still unsure whether we should do this,
            // but it might take more thought to implement, because it'd be hard
            // to tease apart which is the domain and which is the other strand
            // cell. For now I'm going to see what happens if StrandCells don't
            // get a domain value.
            Language::StrandCell => {
                assert_eq!(enode.children.len(), 2);

                Meta {
                    value: match (
                        egraph[enode.children[0]].metadata.value.as_ref(),
                        egraph[enode.children[1]].metadata.value.as_ref(),
                    ) {
                        (Some(Value::DomainValue(v)), None)
                        | (None, Some(Value::DomainValue(v))) => {
                            Some(Value::StrandCellValue(vec![v.clone()]))
                        }
                        (Some(Value::DomainValue(v)), Some(Value::StrandCellValue(s))) => {
                            Some(Value::StrandCellValue(
                                std::iter::once(v).chain(s.iter()).cloned().collect(),
                            ))
                        }
                        (Some(Value::StrandCellValue(s)), Some(Value::DomainValue(v))) => {
                            Some(Value::StrandCellValue(
                                s.iter().chain(std::iter::once(v)).cloned().collect(),
                            ))
                        }
                        (None, None) => Some(Value::StrandCellValue(vec![])),
                        _ => panic!(
                            "Unexpected combination of metadata:\n{:?}\n{:?}",
                            egraph[enode.children[0]].metadata.value.as_ref(),
                            egraph[enode.children[1]].metadata.value.as_ref()
                        ),
                    },
                }
            }
            Language::DoubleStrandCell => {
                assert_eq!(enode.children.len(), 3);

                match (
                    egraph[enode.children[0]].metadata.value.as_ref(),
                    egraph[enode.children[1]].metadata.value.as_ref(),
                    egraph[enode.children[2]].metadata.value.as_ref(),
                ) {
                    (
                        Some(Value::StrandCellValue(bottom)),
                        Some(Value::StrandCellValue(top)),
                        None,
                    )
                    | (
                        None,
                        Some(Value::StrandCellValue(bottom)),
                        Some(Value::StrandCellValue(top)),
                    )
                    | (
                        Some(Value::DoubleStrandCellValue(_)),
                        Some(Value::StrandCellValue(bottom)),
                        Some(Value::StrandCellValue(top)),
                    )
                    | (
                        Some(Value::StrandCellValue(bottom)),
                        Some(Value::StrandCellValue(top)),
                        Some(Value::DoubleStrandCellValue(_)),
                    ) => {
                        // Check that we have two complementary strand cells as arguments.
                        println!("{:?}", bottom);
                        assert!(bottom
                            .iter()
                            .map(|d: &Domain| {
                                match d {
                                    Domain::Long(v) => {
                                        Domain::Long(DomainId::Complement(Box::new(v.clone())))
                                    }
                                    Domain::Toehold(v) => {
                                        Domain::Toehold(DomainId::Complement(Box::new(v.clone())))
                                    }
                                }
                            })
                            .rev()
                            .eq(top.iter().cloned()));
                        Meta {
                            value: Some(Value::DoubleStrandCellValue(bottom.clone())),
                        }
                    }
                    _ => panic!(),
                }
            }
        }
    }
}

/// Strand values should be in bottom strand direction.
/// TODO(gus) put this in terms of 3' or 5' ends
/// Returns the id of the strand.
pub fn add_strand_to_egraph(
    egraph: &mut EGraph<Language, Meta>,
    strand_values: &Vec<Domain>,
) -> Id {
    fn add_domain_id_to_egraph(egraph: &mut EGraph<Language, Meta>, domain_id: &DomainId) -> Id {
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
                let toehold_eclass_id: Id = egraph.add(ENode::new(
                    Language::ToeholdDomain,
                    vec![domain_id_enode_id],
                ));
                egraph.add(ENode::new(Language::Domain, vec![toehold_eclass_id]))
            }
            &Domain::Long(id) => {
                let domain_id_enode_id: Id = add_domain_id_to_egraph(egraph, id);
                let long_eclass_id: Id =
                    egraph.add(ENode::new(Language::LongDomain, vec![domain_id_enode_id]));
                egraph.add(ENode::new(Language::Domain, vec![long_eclass_id]))
            }
        }
    }

    let nil_eclass_id = egraph.add(ENode::leaf(Language::Nil));

    let domain_eclass_ids: Vec<Id> = strand_values
        .iter()
        // First, add every domain into the egraph, returning a list of eclass
        // IDs of the added domains.
        .map(|domain: &Domain| add_domain_to_egraph(egraph, domain))
        .collect();

    let id: Id = domain_eclass_ids
        .iter()
        // Then, over the list of domain IDs, we construct a tree of strand
        // cells, which for domains [d0, d1, d2] will look something like:
        //         c
        //        / \
        //       c  d2
        //      / \   \
        //     c  d1  ...
        //    / \   \
        //  nil d0  ...
        //       |
        //      ...
        // Where each c is a strand cell and each d is a domain node.
        .fold(
            nil_eclass_id,
            |previous_strand_cell_eclass_id: Id, domain_eclass_id: &Id| {
                egraph.add(ENode::new(
                    Language::StrandCell,
                    vec![previous_strand_cell_eclass_id, *domain_eclass_id],
                ))
            },
        );

    egraph.rebuild();

    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use egg::{EGraph, Pattern, Searcher};

    #[test]
    fn add_to_egraph() {
        let mut egraph = EGraph::<Language, Meta>::default();
        let id: Id = add_strand_to_egraph(
            &mut egraph,
            &vec![
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                Domain::Long(DomainId::DomainId(3)),
            ],
        );

        //egraph.dot().to_svg("add-to-egraph.svg").unwrap();

        assert_eq!(
            "(strand-cell
              (strand-cell
               (strand-cell
                (strand-cell
                 (strand-cell
                  nil
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
            .search(&egraph)
            .len(),
            1
        );

        assert_eq!(
            match egraph[id].metadata.value.as_ref().unwrap() {
                Value::StrandCellValue(s) => s,
                _ => panic!(),
            },
            &[
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                Domain::Long(DomainId::DomainId(3)),
            ],
        )
    }
}
