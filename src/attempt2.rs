use egg::{define_language, EGraph};

type DomainId = u32;
type StrandId = u32;

// enum Domain {
//     LongDomain(DomainId),
//     ToeholdDomain(DomainId),
// }

// struct StrandCell {
//     domain: Domain,
//     previous: Option(Rc<StrandCell>),
//     next: Option(Rc<StrandCell>),
// }

// struct Strand {
//     strand_start: Rc<StrandCell>,
// }

define_language! {
    enum Language {
        // Syntax:
        // a unique strand: (strand <strand-id> <strand-cell>...)
        Strand = "strand",

        // strand-cell: (strand-cell <domain> prev: [<strand-cell> | nil]
        //                                    next: [<strand-cell> | nil])
        StrandCell = "strand-cell",

        // domain: [(long-domain <domain-id>)
        //          | (toehold-domain <domain-id>)]
        LongDomain = "long-domain",
        ToeholdDomain = "toehold-domain",

        Nil = "nil",
        DomainId(DomainId),
        StrandId(StrandId),
    }
}

fn _add_strand_to_egraph(
    _egraph: &mut EGraph<Language, ()>,
    _strand_id: StrandId,
    strand_values: &Vec<DomainId>,
) {
    strand_values
        .iter()
        .rev()
        .fold((), |_acc: (), _domain: &DomainId| {
            // This is where I realized this wouldn't work.
            // You can't (or, it's very annoying to) construct a doubly linked list
            // in the egraph!
        });
}
