//! Reading and checking sequence diagrams (docs/SPEC.md, Sequence): a spec
//! names its kind, a sequence reads as one, and each of its rules reports
//! its problem where it is.

use archgram_core::sequence::{Operator, SequenceStill, What};
use archgram_core::{Diagram, SpecError, parse, parse_spec};

/// A sequence with the given participants and messages spliced in.
fn sequence(participants: &str, messages: &str) -> String {
    format!(
        r#"{{ "archgram": 1, "diagram": "sequence", "title": "t", "description": "d",
  "participants": [{participants}], "messages": [{messages}] }}"#
    )
}

const THREE: &str = r#"{ "id": "browser", "kind": "browser", "label": "Browser" },
  { "id": "api", "kind": "service", "label": "API" },
  { "id": "db", "kind": "database", "label": "Users" }"#;

/// The sign-in of docs/SPEC.md (Sequence), sources left out.
const SIGN_IN: &str = r#"
  { "from": "browser", "to": "api", "label": "POST /login" },
  { "from": "api", "to": "db", "label": "find user" },
  { "reply": "db", "label": "user" },
  { "alt": [
      { "when": "password matches", "messages": [ { "reply": "api", "label": "session" } ] },
      { "when": "else", "messages": [ { "reply": "api", "label": "401", "refused": true } ] }
  ] }"#;

fn errors(json: &str) -> Vec<SpecError> {
    parse(json).expect_err("the spec should be refused")
}

/// Each problem as `pointer: message`.
fn said(json: &str) -> Vec<String> {
    errors(json).iter().map(ToString::to_string).collect()
}

fn one(json: &str) -> String {
    let said = said(json);
    assert_eq!(said.len(), 1, "{said:#?}");
    said[0].clone()
}

#[test]
fn a_sequence_reads_as_one() {
    let Diagram::Sequence(seq) = parse(&sequence(THREE, SIGN_IN)).expect("sign in") else {
        panic!("read as another kind")
    };
    assert_eq!(seq.participants.len(), 3);
    assert_eq!(seq.message_count(), 5);
    assert_eq!(seq.still, SequenceStill::Numbers);
    match seq.messages[3].what() {
        Ok(What::Fragment { operator, operands }) => {
            assert_eq!(operator, Operator::Alt);
            assert_eq!(operands.len(), 2);
        }
        other => panic!("{other:?}"),
    }
}

/// A spec that names no kind is an architecture diagram, as every spec
/// written before `diagram` is, and so is one that names it.
#[test]
fn a_spec_without_a_kind_is_an_architecture_diagram() {
    let nodes = r#"{ "archgram": 1, "title": "t", "description": "d",
      "nodes": [ { "id": "api", "kind": "service", "label": "API" } ] }"#;
    assert!(matches!(parse(nodes), Ok(Diagram::Architecture(_))));
    let named = nodes.replacen("\"title\"", "\"diagram\": \"architecture\", \"title\"", 1);
    assert!(matches!(parse(&named), Ok(Diagram::Architecture(_))));
    assert!(parse_spec(&named).is_ok());
}

#[test]
fn an_unknown_kind_is_refused_where_it_is_written() {
    let text = sequence(THREE, SIGN_IN).replace("\"sequence\"", "\"flow\"");
    let e = one(&text);
    assert!(e.starts_with("1:"), "{e}");
    assert!(e.contains("unknown variant `flow`"), "{e}");
    assert!(
        e.contains("`architecture`") && e.contains("`sequence`"),
        "{e}"
    );
}

/// A sequence takes a sequence's fields: an architecture spec's are
/// refused, `direction` among them, since time runs down the page.
#[test]
fn a_sequence_refuses_an_architecture_spec_s_fields() {
    for field in [
        r#""direction": "down""#,
        r#""nodes": []"#,
        r#""legend": true"#,
    ] {
        let text =
            sequence(THREE, SIGN_IN).replacen("\"title\"", &format!("{field}, \"title\""), 1);
        let e = one(&text);
        assert!(e.contains("unknown field"), "{field}: {e}");
    }
}

#[test]
fn parse_spec_reads_architecture_alone() {
    let e = parse_spec(&sequence(THREE, SIGN_IN)).expect_err("a sequence");
    assert_eq!(e[0].location.to_string(), "/diagram");
}

#[test]
fn a_message_names_participants_that_exist() {
    assert_eq!(
        one(&sequence(
            THREE,
            r#"{ "from": "browser", "to": "apx" }, { "from": "api", "to": "db" }"#
        )),
        "/messages/0/to: no participant has the id `apx`; did you mean `api`?"
    );
}

#[test]
fn every_participant_takes_part() {
    assert_eq!(
        one(&sequence(THREE, r#"{ "from": "browser", "to": "api" }"#)),
        "/participants/2: participant `db` is in no message; take it out, or add the messages it takes part in"
    );
}

#[test]
fn a_sequence_has_two_participants_and_a_message() {
    let alone = sequence(
        r#"{ "id": "api", "kind": "service", "label": "API" }"#,
        r#"{ "from": "api", "to": "api" }"#,
    );
    assert!(one(&alone).starts_with("/participants: a sequence needs at least two participants"));
    let quiet = sequence(THREE, "");
    let said = said(&quiet);
    assert!(
        said.iter()
            .any(|e| e.starts_with("/messages: a sequence needs at least one message")),
        "{said:#?}"
    );
}

#[test]
fn participant_ids_are_valid_and_unique() {
    let twice = r#"{ "id": "api", "kind": "service", "label": "API" }, { "id": "api", "kind": "database", "label": "DB" }"#;
    assert_eq!(
        one(&sequence(twice, r#"{ "from": "api", "to": "api" }"#)),
        "/participants/1/id: the id `api` is used by two participants"
    );
}

/// An item is one of a message, a reply and a fragment, with that form's
/// fields alone.
#[test]
fn an_item_is_a_message_a_reply_or_a_fragment() {
    let cases = [
        (
            r#"{ "label": "lost" }"#,
            "/messages/0: an item is a message",
        ),
        (
            r#"{ "from": "browser" }"#,
            "/messages/0: a message needs `to`",
        ),
        (r#"{ "to": "api" }"#, "/messages/0: a message needs `from`"),
        (
            r#"{ "from": "api", "reply": "api" }"#,
            "/messages/0: a reply names who replies in `reply`, not in `from`",
        ),
        (
            r#"{ "reply": "api", "async": true }"#,
            "/messages/0: a reply does not wait",
        ),
        (
            r#"{ "from": "api", "to": "db", "opt": { "messages": [] } }"#,
            "/messages/0: a fragment is an item of its own",
        ),
        (
            r#"{ "label": "x", "opt": { "messages": [] } }"#,
            "/messages/0: a fragment holds operands only",
        ),
        (
            r#"{ "opt": { "messages": [] }, "loop": { "messages": [] } }"#,
            "/messages/0: an item holds one fragment",
        ),
    ];
    for (item, start) in cases {
        let messages = format!(
            r#"{item}, {{ "from": "browser", "to": "api" }}, {{ "from": "api", "to": "db" }}"#
        );
        let said = said(&sequence(THREE, &messages));
        assert!(
            said.iter().any(|e| e.starts_with(start)),
            "{item}: {said:#?}"
        );
    }
}

/// A reply answers the latest call to its participant still waiting, and
/// goes back to that call's caller.
#[test]
fn a_reply_answers_a_call_still_waiting() {
    let none = r#"{ "from": "browser", "to": "api" }, { "reply": "api" }, { "reply": "api" }, { "from": "api", "to": "db" }"#;
    assert_eq!(
        one(&sequence(THREE, none)),
        "/messages/2/reply: `api` replies, but no call to `api` is waiting for a reply before it"
    );
    let elsewhere = r#"{ "from": "browser", "to": "api" }, { "from": "api", "to": "db" }, { "reply": "db", "to": "browser" }, { "reply": "api" }"#;
    assert!(
        one(&sequence(THREE, elsewhere)).starts_with("/messages/2/to: the reply from `db` answers `api`'s call, so it goes to `api`, not `browser`")
    );
    // A send does not wait, and a refused call is answered by its refusal.
    let sent = r#"{ "from": "browser", "to": "api", "async": true }, { "from": "api", "to": "db", "refused": true }, { "reply": "db" }"#;
    let said = said(&sequence(THREE, sent));
    assert!(
        said.iter()
            .any(|e| e.starts_with("/messages/2/reply: `db` replies, but no call")),
        "{said:#?}"
    );
}

/// After an `alt`, a call left waiting by any of its operands may still be
/// answered; inside it, each operand starts from the calls waiting before it.
#[test]
fn an_alt_s_operands_each_start_from_the_calls_waiting() {
    assert!(parse(&sequence(THREE, SIGN_IN)).is_ok());
    let later = r#"{ "from": "browser", "to": "api" },
      { "alt": [ { "when": "cached", "messages": [ { "reply": "api" } ] },
                 { "when": "else", "messages": [ { "from": "api", "to": "db" }, { "reply": "db" } ] } ] },
      { "reply": "api" }"#;
    assert!(
        parse(&sequence(THREE, later)).is_ok(),
        "{:#?}",
        said(&sequence(THREE, later))
    );
}

#[test]
fn an_alt_s_operands_carry_guards_else_last() {
    let alt = |a: &str, b: &str| {
        sequence(
            THREE,
            &format!(
                r#"{{ "from": "browser", "to": "api" }}, {{ "alt": [ {{ {a} "messages": [ {{ "from": "api", "to": "db" }} ] }}, {{ {b} "messages": [ {{ "reply": "api" }} ] }} ] }}"#
            ),
        )
    };
    assert!(parse(&alt(r#""when": "ok","#, r#""when": "else","#)).is_ok());
    assert!(parse(&alt(r#""when": "ok","#, r#""when": "not ok","#)).is_ok());
    assert_eq!(
        one(&alt("", r#""when": "else","#)),
        "/messages/1/alt/0: each operand of `alt` needs a guard in `when`; the last may be `else`"
    );
    assert_eq!(
        one(&alt(r#""when": "else","#, r#""when": "ok","#)),
        "/messages/1/alt/0/when: `else` goes on the last operand of `alt`"
    );
    assert_eq!(
        one(&alt(r#""when": " ","#, r#""when": "else","#)),
        "/messages/1/alt/0/when: the guard is empty; say when the operand happens, or leave `when` out"
    );
}

#[test]
fn fragments_take_their_operator_s_operands() {
    let item = |fragment: &str| {
        sequence(
            THREE,
            &format!(
                r#"{{ "from": "browser", "to": "api" }}, {{ "from": "api", "to": "db" }}, {fragment}"#
            ),
        )
    };
    let solo = r#"{ "alt": [ { "when": "ok", "messages": [ { "reply": "db" } ] } ] }"#;
    assert!(one(&item(solo)).starts_with("/messages/2/alt: `alt` needs two operands or more"));
    let par = r#"{ "par": [ { "when": "x", "messages": [ { "from": "db", "to": "api", "async": true } ] }, { "messages": [ { "from": "api", "to": "browser", "async": true } ] } ] }"#;
    assert_eq!(
        one(&item(par)),
        "/messages/2/par/0/when: the operands of `par` all happen, so they take no guard"
    );
    let opt = r#"{ "opt": { "when": "else", "messages": [ { "reply": "db" } ] } }"#;
    assert_eq!(
        one(&item(opt)),
        "/messages/2/opt/when: `else` belongs to `alt`; `opt` has one operand"
    );
    let empty = r#"{ "loop": { "when": "each row", "messages": [] } }"#;
    assert_eq!(
        one(&item(empty)),
        "/messages/2/loop/messages: an operand of `loop` holds no messages"
    );
}

/// A misspelt field is refused where it is written, inside an operand too.
#[test]
fn a_misspelt_field_is_refused_at_its_line() {
    let text = sequence(THREE, SIGN_IN).replacen("\"when\": \"else\"", "\"wen\": \"else\"", 1);
    let e = one(&text);
    assert!(e.contains("unknown field `wen`"), "{e}");
    assert!(!e.starts_with('/'), "a line and column: {e}");
}

/// Each participant and message names its sources at its own pointer, in
/// an operand too, for `check`.
#[test]
fn sources_are_found_in_operands() {
    let messages = SIGN_IN.replacen(
        r#"{ "reply": "api", "label": "session" }"#,
        r#"{ "reply": "api", "label": "session", "source": "../../src/api.ts#res.json(session" }"#,
        1,
    );
    let participants = THREE.replacen(
        r#""label": "API" }"#,
        r#""label": "API", "source": "../../src/api.ts" }"#,
        1,
    );
    let Ok(Diagram::Sequence(seq)) = parse(&sequence(&participants, &messages)) else {
        panic!("{:#?}", said(&sequence(&participants, &messages)))
    };
    let named: Vec<(String, String)> = seq
        .owned()
        .into_iter()
        .filter(|(_, _, s)| s.is_some())
        .map(|(pointer, owner, _)| (pointer, owner))
        .collect();
    assert_eq!(
        named,
        [
            ("/participants/1/source".into(), "participant api".into()),
            (
                "/messages/3/alt/0/messages/0/source".into(),
                "reply from api".into()
            ),
        ]
    );
}

#[test]
fn a_source_in_a_sequence_keeps_its_form() {
    let messages = SIGN_IN.replacen(
        r#""label": "find user" }"#,
        r#""label": "find user", "source": "/etc/passwd" }"#,
        1,
    );
    let e = one(&sequence(THREE, &messages));
    assert!(
        e.starts_with("/messages/1/source: message api \u{2192} db: "),
        "{e}"
    );
}

// Drawing: the layout keeps a sequence readable, and the SVG says what
// the drawing shows.

use archgram_core::geometry::Rect;
use archgram_core::layout::sequence::{Sort, place};
use archgram_core::logos::NoLogos;
use archgram_core::render::Options;

fn example(name: &str) -> String {
    let path = format!("{}/../../examples/{name}.json", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn read(name: &str) -> archgram_core::sequence::Sequence {
    match parse(&example(name)) {
        Ok(Diagram::Sequence(seq)) => seq,
        other => panic!("{name}: {other:?}"),
    }
}

/// Whether `inner` lies within `outer`.
fn within(inner: Rect, outer: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

#[test]
fn messages_run_down_the_page_one_row_each() {
    for name in ["sequence-sign-in", "sequence-oauth"] {
        let layout = place(&read(name), &NoLogos);
        let ys: Vec<f64> = layout.messages.iter().map(|m| m.path[0].y).collect();
        assert!(ys.windows(2).all(|w| w[0] < w[1]), "{name}: {ys:?}");
        let numbers: Vec<u32> = layout.messages.iter().map(|m| m.number).collect();
        let last = u32::try_from(numbers.len()).expect("a few messages");
        assert_eq!(numbers, (1..=last).collect::<Vec<_>>());
        let (top, bottom) = layout.lifeline_span;
        assert!(ys.iter().all(|&y| y > top && y < bottom), "{name}");
    }
}

#[test]
fn labels_pills_and_heads_never_overlap() {
    for name in ["sequence-sign-in", "sequence-oauth"] {
        let layout = place(&read(name), &NoLogos);
        let mut boxes: Vec<(String, Rect)> = layout
            .heads
            .iter()
            .enumerate()
            .map(|(i, &r)| (format!("head {i}"), r))
            .collect();
        for m in &layout.messages {
            if let Some((_, r)) = &m.label {
                boxes.push((format!("label {}", m.number), *r));
            }
            if let Some(p) = m.pill {
                boxes.push((format!("pill {}", m.number), p));
            }
        }
        for (i, (a, ra)) in boxes.iter().enumerate() {
            for (b, rb) in &boxes[i + 1..] {
                assert!(!ra.overlaps(rb), "{name}: {a} {ra:?} overlaps {b} {rb:?}");
            }
        }
    }
}

/// A fragment's frame holds its tag, its guards, every message in it and
/// every fragment nested in it.
#[test]
fn a_fragment_holds_what_it_frames() {
    let layout = place(&read("sequence-oauth"), &NoLogos);
    assert_eq!(layout.fragments.len(), 3, "par, loop and the alt inside it");
    for f in &layout.fragments {
        assert!(within(f.tag, f.frame), "{f:?}");
        for (_, g) in &f.guards {
            assert!(within(*g, f.frame), "{f:?}");
        }
        for m in &layout.messages {
            let inside = m
                .path
                .iter()
                .all(|p| p.y > f.frame.y && p.y < f.frame.bottom());
            if inside {
                for p in &m.path {
                    assert!(
                        p.x >= f.frame.x && p.x <= f.frame.right(),
                        "message {} leaves {f:?}",
                        m.number
                    );
                }
            }
        }
    }
    let (looped, alt) = (&layout.fragments[1], &layout.fragments[2]);
    assert!(
        within(alt.frame, looped.frame),
        "the alt sits inside the loop"
    );
}

#[test]
fn a_call_a_send_and_a_reply_are_drawn_as_uml_draws_them() {
    let svg = archgram_core::build(&example("sequence-oauth"), Options::default()).expect("draws");
    let layout = place(&read("sequence-oauth"), &NoLogos);
    let calls = layout
        .messages
        .iter()
        .filter(|m| m.sort == Sort::Call && !m.refused)
        .count();
    let replies = layout
        .messages
        .iter()
        .filter(|m| m.sort == Sort::Reply)
        .count();
    assert_eq!(svg.matches(r#"class="arrowhead filled""#).count(), calls);
    assert_eq!(svg.matches(r#"class="edge reply""#).count(), replies);
    assert_eq!(
        svg.matches(r#"class="refused-mark""#).count(),
        2,
        "the ✕ and its arrowhead"
    );
    assert!(svg.contains(r#"class="refused-head""#));
    for op in ["alt", "loop", "par"] {
        assert!(svg.contains(&format!(">{op}</text>")), "{op}");
    }
    assert!(svg.contains("[for each page of the profile]"));
}

#[test]
fn a_screen_reader_hears_each_message_in_order() {
    let svg =
        archgram_core::build(&example("sequence-sign-in"), Options::default()).expect("draws");
    let desc = svg
        .split("<desc id=\"desc\">")
        .nth(1)
        .and_then(|d| d.split("</desc>").next())
        .expect("a description");
    assert!(
        desc.ends_with(
            "1. Browser calls API: POST /login. 2. API calls Users: find user. 3. Users replies to API: user. If password matches: 4. API replies to Browser: session. Otherwise: 5. API replies to Browser: 401, refused."
        ),
        "{desc}"
    );
}

#[test]
fn a_sequence_draws_the_same_bytes_every_time() {
    let spec = example("sequence-oauth");
    let first = archgram_core::build(&spec, Options::default()).unwrap();
    for _ in 0..3 {
        assert_eq!(
            archgram_core::build(&spec, Options::default()).unwrap(),
            first
        );
    }
}

/// `still: none` leaves the numbers out.
#[test]
fn still_none_draws_no_numbers() {
    let spec =
        example("sequence-sign-in").replacen("\"title\"", "\"still\": \"none\", \"title\"", 1);
    let svg = archgram_core::build(&spec, Options::default()).unwrap();
    assert!(!svg.contains("class=\"step\""));
}
