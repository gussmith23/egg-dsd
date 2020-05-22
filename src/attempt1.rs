use egg::define_language;

type DomainId = u32;

define_language! {
    enum Language {
        // Syntax:
        // a unique strand: (strand <strand-id> <segments>)
        Strand = "strand",

        // segments: (cons <segment> [<segments> | nil])
        Cons = "cons",

        // segment: [(matched-segment <domain>)
        //           | (unmatched-segment <domain>)
        //           | top-nick
        //           | bottom-nick ]
        MatchedSegment = "matched-segment",
        UnmatchedSegment = "unmatched-segment",
        TopNick = "top-nick",
        BottomNick = "bottom-nick",

        // domain: [(long-domain <domain-id>)
        //          | (toehold-domain <domain-id>)]
        LongDomain = "long-domain",
        ToeholdDomain = "toehold-domain",

        Nil = "nil",
        DomainId(DomainId),
        StrandId(String),
    }
}
