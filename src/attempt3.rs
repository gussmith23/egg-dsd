use egg::{define_language, EGraph, ENode, Id};

pub enum DomainId {
    Complement(Box<DomainId>),
    DomainId(u32),
}
type StrandId = u32;

pub enum Domain {
    Toehold(DomainId),
    Long(DomainId),
}

pub enum TopOrBottom {
    Top,
    Bottom,
}

define_language! {
    pub enum Language {
        // Syntax:
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

        // domain-id: [(complement <domain-id>)
        //            | <DomainId>]
        Complement = "complement",
        // TODO(gus) give an alias for u32 here?
        DomainId(u32),

        Nil = "nil",
        StrandId(StrandId),
    }
}

pub fn add_strand_to_egraph(
    egraph: &mut EGraph<Language, ()>,
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
                egraph: &mut EGraph<Language, ()>,
                domain_id: &DomainId,
            ) -> Id {
                match &domain_id {
                    &DomainId::Complement(domain_id) => {
                        let domain_id_egraph_id: Id = add_domain_id_to_egraph(egraph, domain_id);
                        egraph.add(ENode::new(Language::Complement, vec![domain_id_egraph_id]))
                    }
                    &DomainId::DomainId(id) => egraph.add(ENode::leaf(Language::DomainId(*id))),
                }
            }
            fn add_domain_to_egraph(egraph: &mut EGraph<Language, ()>, domain: &Domain) -> Id {
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
        let mut egraph = EGraph::<Language, ()>::default();
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
        let mut egraph = EGraph::<Language, ()>::default();
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
    use egg::{rewrite, Rewrite};

    pub fn simplify_double_complement() -> Rewrite<Language, ()> {
        rewrite!(
        "simplify-double-complement";
        "(complement (complement ?a))" =>
            "?a")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use egg::{EGraph, ENode, Runner};

        #[test]
        fn simplify_double_complement() {
            let mut egraph = EGraph::default();

            let enode_id = egraph.add(ENode::leaf(Language::DomainId(0)));
            let enode_id = egraph.add(ENode::new(Language::Complement, vec![enode_id]));
            let enode_id = egraph.add(ENode::new(Language::Complement, vec![enode_id]));

            assert!(!egraph[enode_id]
                .nodes
                .iter()
                .any(|enode| { enode.op == Language::DomainId(0) }));
            let runner = Runner::new()
                .with_egraph(egraph)
                .run(&[super::simplify_double_complement()]);
            assert!(runner.egraph[enode_id]
                .nodes
                .iter()
                .any(|enode| { enode.op == Language::DomainId(0) }));
        }
    }
}
