use egg::{define_language, EGraph};
use std::vec::Vec;

fn main() {
    let expr = "
     a"
    .parse()
    .unwrap();

    use egg_dsd::attempt0::Language;
    let (mut egraph, _id) = EGraph::<Language, ()>::from_expr(&expr);
    egraph.dot().to_svg("initial-program.svg").unwrap();

    egraph.add_expr(&"b".parse().unwrap());

    egraph.dot().to_svg("initial-program.svg").unwrap();
}

fn _egg_dsd() {
    type DomainId = u32;
    enum Domain {
        // Short, or Toehold, domains
        Toehold(DomainId),
        Long(DomainId),
    };
    // Canonical form = 3' end -> 5' end.
    // TODO(gus) does that even make sense?
    type SingleStrand = Vec<Domain>;
    // A strand, potentially double-stranded, could be

    struct DoubleStrand {
        left_bottom: SingleStrand,
        left_top: SingleStrand,
        // The section which is duplicated, top and bottom.
        middle: SingleStrand,
        right_bottom: SingleStrand,
        right_top: SingleStrand,
    };

    define_language! {
        enum Language {
            SingleStrand = "single-strand",
            // 5 children
            // 1. Left-bottom
            // 2. Left-top
            // 3. Middle
            // 4. Right-bottom
            // 5. Right-top
            DoubleStrand = "double-strand",
            // TODO(gus) or Domain(Domain)?
            //LongDomain = "long-domain",
            //ToeholdDomain = "toehold-domain",
            ToeholdDomain = "toehold-domain",
            LongDomain = "long-domain",
            DomainId(DomainId),
            // A "quantum" of homogeneous solution
            // And its identifier.
            // TODO(gus) god-awful naming choice.
            Droplet = "droplet",
            DropletId(String),
        }
    }

    let expr = "
     (droplet droplet-a
      (double-strand
       (single-strand (long-domain 1) (toehold-domain 2))
       (single-strand (long-domain 3) (long-domain 4))
       (single-strand (long-domain 5) (long-domain 6))
       (single-strand (long-domain 7) (long-domain 8))
       (single-strand (long-domain 9) (long-domain 10))
      )
     )"
    .parse()
    .unwrap();

    let (mut egraph, _id) = EGraph::<Language, ()>::from_expr(&expr);
    egraph.dot().to_svg("initial-program.svg").unwrap();

    egraph.add_expr(
        &"(droplet droplet-b (single-strand (long-domain 1) (toehold-domain 2)))"
            .parse()
            .unwrap(),
    );

    egraph.dot().to_svg("initial-program.svg").unwrap();
}
