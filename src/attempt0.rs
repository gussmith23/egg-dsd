use egg::{
    define_language, Applier, EGraph, ENode, Id, Metadata, Pattern, SearchMatches, Searcher, Subst,
    Var,
};
use std::option::Option;

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

#[derive(Debug, PartialEq, Eq, Clone)]
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

const A: &'static str = "?A";
const B: &'static str = "?B";
struct AnyTwoStrandsSearcher;
impl Searcher<Language, Meta> for AnyTwoStrandsSearcher {
    fn search_eclass(&self, egraph: &EGraph<Language, Meta>, eclass: Id) -> Option<SearchMatches> {
        let p: Pattern<Language> = A.parse().unwrap();
        let matches = p.search(egraph);
        let matches: Vec<Subst> =
            matches
                .iter()
                .fold(vec![], |acc: Vec<Subst>, x: &SearchMatches| {
                    let mut out = acc.clone();
                    out.append(&mut x.substs.clone());
                    out
                });
        let eclass_matches = p.search_eclass(egraph, eclass).unwrap();
        use itertools::Itertools;
        let a_var: Var = A.parse().unwrap();
        let b_var: Var = B.parse().unwrap();
        Some(SearchMatches {
            eclass: eclass,
            substs: eclass_matches
                .substs
                .iter()
                .cartesian_product(matches.iter())
                .map(|(a, b): (&Subst, &Subst)| {
                    let mut s: Subst = Subst::default();
                    s.insert(a_var.clone(), a[&a_var]);
                    s.insert(b_var.clone(), b[&a_var]);
                    s
                })
                .collect(),
        })
    }
}
struct ReactionApplier {
    a: Var,
    b: Var,
}
impl Applier<Language, Meta> for ReactionApplier {
    fn apply_one(
        &self,
        _egraph: &mut EGraph<Language, Meta>,
        _matched_id: Id,
        _subst: &Subst,
    ) -> Vec<Id> {
        println!("{:?}", self.a);
        println!("{:?}", self.b);
        vec![]
    }
}

#[test]
fn any_two_strands_searcher_test() {
    let mut egraph: EGraph<Language, Meta> = EGraph::default();
    let expr = "a".parse().unwrap();
    egraph.add_expr(&expr);
    let expr = "b".parse().unwrap();
    egraph.add_expr(&expr);
    let expr = "c".parse().unwrap();
    egraph.add_expr(&expr);

    let searcher = AnyTwoStrandsSearcher;
    let results = searcher.search(&egraph);
    println!("{:?}", results);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].substs.len(), 3);
    assert_eq!(results[1].substs.len(), 3);
    assert_eq!(results[2].substs.len(), 3);
}

#[test]
fn any_two_strands_test() {
    use egg::{rewrite, Rewrite};
    let rw: Rewrite<Language, Meta> = rewrite! {
        "test";
        AnyTwoStrandsSearcher =>
        {
            ReactionApplier {
                a: A.parse().unwrap(),
                b: B.parse().unwrap(),
            }
        }

    };

    let mut egraph: EGraph<Language, Meta> = EGraph::default();
    let expr = "a".parse().unwrap();
    egraph.add_expr(&expr);
    let expr = "b".parse().unwrap();
    egraph.add_expr(&expr);
    let expr = "c".parse().unwrap();
    egraph.add_expr(&expr);

    let _runner = egg::Runner::new().with_egraph(egraph).run(&[rw]);
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
                    "b" => self::Strand::SingleStrand(vec![]),
                    "c" => self::Strand::SingleStrand(vec![]),
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
