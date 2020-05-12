use egg::{define_language, Applier, EGraph, ENode, Id, Metadata, Subst, Var};

type DomainId = u32;
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
enum Domain {
    // Short, or Toehold, domains
    _Toehold(DomainId),
    _Long(DomainId),
}
// Canonical form = 3' end -> 5' end.
// TODO(gus) does that even make sense?
type SingleStrand = Vec<Domain>;
// A strand, potentially double-stranded, could be

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
struct DoubleStrand {
    left_bottom: SingleStrand,
    left_top: SingleStrand,
    // The section which is duplicated, top and bottom.
    middle: SingleStrand,
    right_bottom: SingleStrand,
    right_top: SingleStrand,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
enum Strand {
    SingleStrand(SingleStrand),
    _DoubleStrand(DoubleStrand),
}

#[derive(Debug, PartialEq, Clone)]
struct Meta {
    strand_value: Strand,
}

define_language! {
    pub enum Language {
        // A strand literal
        Strand(String),
    }
}

struct DSDApplier {
    _a: Var,
}
impl Applier<Language, Meta> for DSDApplier {
    fn apply_one(&self, _egraph: &mut EGraph<Language, Meta>, _id: Id, _subst: &Subst) -> Vec<Id> {
        vec![]
    }
}

#[test]
fn dumb_test() {
    assert!(true);
}

impl Metadata<Language> for Meta {
    type Error = ();

    fn merge(&self, other: &Self) -> Self {
        assert_eq!(self, other);
        self.clone()
    }

    fn make(_egraph: &EGraph<Language, Self>, enode: &ENode<Language>) -> Self {
        use self::Language::*;
        match &enode.op {
            Strand(id) => Meta {
                strand_value: match id.as_str() {
                    "a" => self::Strand::SingleStrand(vec![]),
                    _ => panic!(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expression() {
        let expr = "a".parse().unwrap();
        let (mut egraph, _id) = EGraph::<Language, ()>::from_expr(&expr);
        egraph.add_expr(&"b".parse().unwrap());
    }
}
