use egg::{define_language, EGraph, ENode, Id, Metadata};

//pub mod rewrites;

type DomainIdValue = u32;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainId {
    Complement(Box<DomainId>),
    DomainId(DomainIdValue),
}
//type StrandId = u32;

#[derive(Clone)]
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
        // BottomDoubleStrandCell = "bottom-double-strand-cell",
        // TopDoubleStrandCell = "top-double-strand-cell",

        // TODO(gus) too many variants here?
        // strand-cell: [  (strand-cell)
        //               | (strand-cell <domain>)
        //               | (strand-cell <domain> <domain>)
        //               | (strand-cell <strand-cell> <domain>)
        //               | (strand-cell <domain> <strand-cell>)
        //               | (strand-cell <strand-cell> <strand-cell>) ]
        StrandCell = "strand-cell",

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
            Language::DomainIdValue(id) => Meta {
                domain_id: Some(DomainId::DomainId(*id)),
            },
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
            Language::StrandCell => Meta { domain_id: None },
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

    let empty_strand_cell_id = egraph.add(ENode::new(Language::StrandCell, vec![]));

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
        //   c  d0  ...
        //       |
        //      ...
        // Where each c is a strand cell and each d is a domain node. Note that
        // the leftmost/bottommost cell has no children; this is essentially a
        // "nil" in a cons cell. It just makes the fold easier, and could be
        // changed explicitly to "nil" later on.
        .fold(
            empty_strand_cell_id,
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
        add_strand_to_egraph(
            &mut egraph,
            &vec![
                Domain::Toehold(DomainId::DomainId(0)),
                Domain::Long(DomainId::DomainId(1)),
                Domain::Long(DomainId::DomainId(2)),
                Domain::Long(DomainId::Complement(Box::new(DomainId::DomainId(2)))),
                Domain::Long(DomainId::DomainId(3)),
            ],
        );

        egraph.dot().to_svg("add-to-egraph.svg").unwrap();

        assert_eq!(
            "(strand-cell
              (strand-cell
               (strand-cell
                (strand-cell
                 (strand-cell
                  (strand-cell)
                  (toehold-domain (domain-id 0))
                 )
                 (long-domain (domain-id 1))
                )
                (long-domain (domain-id 2))
               )
               (long-domain (complement (domain-id 2)))
              )
              (long-domain (domain-id 3))
             )
             "
            .parse::<Pattern<_>>()
            .unwrap()
            .search(&egraph)
            .len(),
            1
        );
    }
}
