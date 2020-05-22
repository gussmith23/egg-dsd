use egg::{define_language, EGraph, ENode, Id};

type DomainId = u32;
type StrandId = u32;

enum Domain {
    Complement(Box<Domain>),
    Toehold(DomainId),
    Long(DomainId),
}

enum TopOrBottom {
    Top,
    Bottom,
}

define_language! {
    enum Language {
        // Syntax:
        // a unique strand: (strand <strand-id> <strand-cell>)
        Strand = "strand",

        // strand-cell: [(bottom-strand-cell <domain> [<strand-cell> | nil])
        //               | (top-strand-cell <domain> [<strand-cell> | nil])]
        BottomStrandCell = "bottom-strand-cell",
        TopStrandCell = "top-strand-cell",

        // domain: [(complement <domain>)
        //          | (long-domain)
        //          | (toehold-domain)]
        Complement = "complement",
        LongDomain(DomainId),
        ToeholdDomain(DomainId),

        Nil = "nil",
        //DomainId(DomainId),
        StrandId(StrandId),
    }
}

fn add_strand_to_egraph(
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
            fn add_domain_to_egraph(egraph: &mut EGraph<Language, ()>, domain: &Domain) -> Id {
                match &domain {
                    &Domain::Complement(domain) => {
                        let domain_egraph_id: Id = add_domain_to_egraph(egraph, domain);
                        egraph.add(ENode::new(Language::Complement, vec![domain_egraph_id]))
                    }
                    &Domain::Toehold(id) => egraph.add(ENode::leaf(Language::ToeholdDomain(*id))),
                    &Domain::Long(id) => egraph.add(ENode::leaf(Language::LongDomain(*id))),
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
    egraph.add(ENode::new(
        Language::Strand,
        vec![strand_id_enode_id, first_strand_cell_id],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egg::EGraph;

    #[test]
    fn add_to_egraph() {
        let mut egraph = EGraph::<Language, ()>::default();
        add_strand_to_egraph(
            &mut egraph,
            TopOrBottom::Bottom,
            0,
            &vec![
                Domain::Toehold(0),
                Domain::Long(1),
                Domain::Long(2),
                Domain::Complement(Box::new(Domain::Long(2))),
                Domain::Long(3),
            ],
        );

        //egraph.dot().to_svg("tmp.svg").unwrap();
    }
}
