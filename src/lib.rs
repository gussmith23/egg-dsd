pub mod attempt0 {
    use egg::{define_language, Applier, EGraph, ENode, Id, Metadata, Subst, Var};
    type DomainId = u32;
    #[derive(Debug, PartialEq, Clone)]
    enum Domain {
        // Short, or Toehold, domains
        Toehold(DomainId),
        Long(DomainId),
    }
    // Canonical form = 3' end -> 5' end.
    // TODO(gus) does that even make sense?
    type SingleStrand = Vec<Domain>;
    // A strand, potentially double-stranded, could be

    #[derive(Debug, PartialEq, Clone)]
    struct DoubleStrand {
        left_bottom: SingleStrand,
        left_top: SingleStrand,
        // The section which is duplicated, top and bottom.
        middle: SingleStrand,
        right_bottom: SingleStrand,
        right_top: SingleStrand,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Meta {
        single_strand_value: Option<SingleStrand>,
        double_strand_value: Option<DoubleStrand>,
    }
    impl Metadata<Language> for Meta {
        type Error = ();

        fn merge(&self, other: &Self) -> Self {
            assert_eq!(self, other);
            self.clone()
        }

        fn make(_egraph: &EGraph<Language, Self>, enode: &ENode<Language>) -> Self {
            match &enode.op {
                Language::SingleStrand(id) => Meta {
                    single_strand_value: Some(match id.as_str() {
                        "a" => vec![Domain::Toehold(1), Domain::Long(2)],
                        _ => panic!(),
                    }),
                    double_strand_value: None,
                },
                Language::DoubleStrand(id) => Meta {
                    single_strand_value: None,
                    double_strand_value: Some(match id.as_str() {
                        "b" => DoubleStrand {
                            left_bottom: vec![Domain::Toehold(1), Domain::Long(2)],
                            left_top: vec![Domain::Toehold(1), Domain::Long(2)],
                            // The section which is duplicated, top and bottom.
                            middle: vec![Domain::Toehold(1), Domain::Long(2)],
                            right_bottom: vec![Domain::Toehold(1), Domain::Long(2)],
                            right_top: vec![Domain::Toehold(1), Domain::Long(2)],
                        },
                        _ => panic!(),
                    }),
                },
            }
        }
    }

    define_language! {
        pub enum Language {
            SingleStrand(String),
            // 5 children
            // 1. Left-bottom
            // 2. Left-top
            // 3. Middle
            // 4. Right-bottom
            // 5. Right-top
            DoubleStrand(String),
        }
    }

    struct DSDApplier {
        a: Var,
    }
    impl Applier<Language, Meta> for DSDApplier {
        fn apply_one(
            &self,
            egraph: &mut EGraph<Language, Meta>,
            _id: Id,
            subst: &Subst,
        ) -> Vec<Id> {
            vec![]
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
}
