use std::str::FromStr;

use nom::Parser as _;
use nom::branch::*;
use nom::bytes::complete::{take_until, take_while};
use nom::bytes::{tag_no_case as tag, take};
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::{Context, context};
use nom::multi::*;
use nom::sequence::*;
use nom_language::error::{VerboseError, convert_error};

use crate::structure::*;

type IResult<'a, T> = nom::IResult<&'a str, T, VerboseError<&'a str>>;

// trait alias because real ones aren't a thing: https://github.com/rust-lang/rust/issues/41517
trait Parser<'a, O>: nom::Parser<&'a str, Output = O, Error = VerboseError<&'a str>> + Sized {
    fn context(self, c: &'static str) -> Context<Self> {
        context(c, self)
    }
}
impl<'a, P, O> Parser<'a, O> for P where
    P: nom::Parser<&'a str, Output = O, Error = VerboseError<&'a str>>
{
}

pub fn parse(out: &str) -> Vec<Beast> {
    many0(beast_with_pretext.context("beast"))
        .parse_complete(out)
        .map_err(|e| e.map(|e| panic!("{}", convert_error(out, e))))
        .unwrap()
        .1
}

fn dbg_dmp<'a, O>(mut parser: impl Parser<'a, O>, context: &'static str) -> impl Parser<'a, O> {
    move |i| match parser.parse_complete(i) {
        Ok(o) => Ok(o),
        Err(e) => {
            eprintln!("{context}({e:#}): {i}");
            Err(e)
        }
    }
}

fn number(i: &str) -> IResult<u64> {
    digit1.map_res(u64::from_str).parse_complete(i)
}

fn leading_ws<'a, O>(parser: impl Parser<'a, O>) -> impl Parser<'a, O> {
    preceded(take_while(char::is_whitespace), parser)
}

fn number_in_feet(s: &str) -> IResult<u64> {
    terminated(
        number,
        leading_ws(alt((tag("feet"), terminated(tag("ft"), opt(char('.')))))),
    )
    .parse_complete(s)
}

fn named_value<'a, O>(name: &'static str, parser: impl Parser<'a, O>) -> impl Parser<'a, O> {
    preceded((tag(name), leading_ws(char(':'))), leading_ws(parser)).context(name)
}

fn named_list<'a, O>(name: &'a str, parser: impl Parser<'a, O>) -> impl Parser<'a, Vec<O>> {
    preceded(
        (tag(name), leading_ws(char(':'))),
        leading_ws(separated_list0(leading_ws(char(',')), leading_ws(parser))),
    )
}

fn parenthesized<'a, T>(value: impl Parser<'a, T>) -> impl Parser<'a, T> {
    delimited(leading_ws(char('(')), value, leading_ws(char(')')))
}

fn named_value_with_paren<'a, T, P>(
    name: &'static str,
    value: impl Parser<'a, T>,
    parenthesized_value: impl Parser<'a, P>,
) -> impl Parser<'a, (T, P)> {
    named_value(
        name,
        (leading_ws(value), parenthesized(parenthesized_value)),
    )
}

fn beast_with_pretext(s: &str) -> IResult<Beast> {
    let (s, (title, tier, role, size, kind)) = find_head(s)?;
    let (s, (attributes, defenses)) = attributes(s)?;
    let (s, health) = leading_ws(health).parse_complete(s)?;
    // Focus: 3
    let (s, focus) = leading_ws(named_value("Focus", number)).parse_complete(s)?;
    // Investiture: 0
    let (s, investiture) = leading_ws(named_value("investiture", number)).parse_complete(s)?;
    let (s, deflect) = leading_ws(opt(named_value_with_paren("Deflect", number, into(alpha1))))
        .parse_complete(s)?;
    let (s, movement) = leading_ws(named_value("Movement", movement)).parse_complete(s)?;
    let (s, senses) = leading_ws(named_value_with_paren(
        "Senses",
        number_in_feet,
        into(alpha1),
    ))
    .parse_complete(s)?;
    let (s, immunities) = leading_ws(opt(named_value(
        "Immunities",
        separated_list0(leading_ws(char(',')), leading_ws(into(alpha1))),
    )))
    .map(|o| o.unwrap_or_else(Vec::new))
    .parse_complete(s)?;
    let skills = |name| {
        leading_ws(opt(named_value(
            name,
            separated_list0(leading_ws(char(',')), leading_ws(skill)),
        )))
        .map(|o| o.unwrap_or_else(Vec::new))
    };
    let (s, physical_skills) = skills("Physical Skills").parse_complete(s)?;
    let (s, cognitive_skills) = skills("Cognitive Skills").parse_complete(s)?;
    let (s, spiritual_skills) = skills("Spiritual Skills").parse_complete(s)?;
    let (s, surge_skills) = skills("Surge Skills").parse_complete(s)?;
    let (s, languages) = opt(leading_ws(named_list(
        "Languages",
        into(recognize(many1(leading_ws(verify(
            take_while(|c: char| !c.is_whitespace()),
            |s: &str| s != "features",
        ))))),
    )))
    .parse_complete(s)?;

    let (s, features) = leading_ws(opt(preceded(
        tag("features"),
        many0(leading_ws(verify(dbg_dmp(feature, "feature"), |f| {
            f.name.trim() != "actions"
        }))),
    )))
    .parse_complete(s)?;

    let (s, actions) =
        leading_ws(opt(preceded(tag("actions"), many0(leading_ws(action))))).parse_complete(s)?;

    let (s, opportunities_and_complications) =
        opt(leading_ws(opportunities_and_complications)).parse_complete(s)?;

    Ok((s, Beast {
        title,
        tier,
        role,
        size,
        kind,
        attributes,
        defenses,
        health,
        focus,
        investiture,
        deflect,
        movement,
        senses,
        immunities,
        physical_skills,
        cognitive_skills,
        spiritual_skills,
        surge_skills,
        languages,
        features: features.unwrap_or_default(),
        actions: actions.unwrap_or_default(),
        opportunities_and_complications,
        image: None,
        description: None,
        tactics: None,
    }))
}

/// ```text
/// Health: 22 (18–26)
/// ```
fn health(s: &str) -> IResult<Ranged> {
    preceded(
        tag("Health:"),
        (
            leading_ws(number),
            delimited(
                leading_ws(char('(')),
                separated_pair(
                    leading_ws(number),
                    leading_ws(one_of("-–")),
                    leading_ws(number),
                ),
                leading_ws(char(')')),
            ),
        ),
    )
    .context("health")
    .map(|(value, (min, max))| Ranged { value, min, max })
    .parse_complete(s)
}

/// ```text
/// 15 ft., swim 30 ft. (30 ft. while Investiture is 0)
/// ```
fn movement(s: &str) -> IResult<Movement> {
    (
        leading_ws(number_in_feet),
        many0(preceded(
            leading_ws(char(',')),
            (leading_ws(into(alpha1)), leading_ws(number_in_feet)).map(|(a, b)| (b, a)),
        )),
        opt(parenthesized(into(take_until(")")))),
    )
        .map(|(value, extra, comment)| Movement {
            value,
            extra,
            comment,
        })
        .parse_complete(s)
}

/// ```text
///    Physical     Cognitive     Spiritual
/// str  def  spd  int  def  wil  awa  def  pre
/// 1  14  3  0  11  1  3  16  3
/// ```
fn attributes(s: &str) -> IResult<(Attributes, Defenses)> {
    let (
        s,
        (
            _ignored_attributes_header,
            (
                strength,
                physical_defense,
                speed,
                intellect,
                cognitive_defense,
                willpower,
                awareness,
                spiritual_defense,
                presence,
            ),
        ),
    ) = many_till(
        anychar.map(|_| ()),
        (
            leading_ws(number),
            leading_ws(number),
            leading_ws(number),
            leading_ws(number),
            leading_ws(number),
            leading_ws(number),
            leading_ws(number),
            leading_ws(number),
            leading_ws(number),
        ),
    )
    .context("attributes")
    .parse_complete(s)?;
    Ok((
        s,
        (
            Attributes {
                strength,
                speed,
                intellect,
                willpower,
                awareness,
                presence,
            },
            Defenses {
                physical_defense,
                cognitive_defense,
                spiritual_defense,
            },
        ),
    ))
}

/// ```text
/// [...]
/// Swordmaster Ardent
/// Tier 1 Rival – Medium Humanoid
/// ```
fn find_head(s: &str) -> IResult<(String, u64, Role, Size, BeastKind)> {
    let (s, (_ignored_pretext, (title, _newline, tier, _space, role))) = many_till(
        anychar.map(|_| ()),
        (into(title), multispace1, tier, space1, role),
    )
    .context("find beast")
    .parse_complete(s)?;
    let (s, _dash) = (multispace0, one_of("–-"), multispace0)
        .context("dash before size")
        .parse_complete(s)?;
    let (s, size) = alpha1
        .map_res(Size::from_str)
        .context("size")
        .parse_complete(s)?;
    let (s, kind) = leading_ws(alpha1.map_res(BeastKind::from_str))
        .context("kind")
        .parse_complete(s)?;
    Ok((s, (title, tier, role, size, kind)))
}

fn title(i: &str) -> IResult<&str> {
    not_line_ending(i)
}

fn tier(i: &str) -> IResult<u64> {
    (tag("Tier"), space1, number)
        .map(|(_, _, d)| d)
        .parse_complete(i)
}

fn role(i: &str) -> IResult<Role> {
    alpha1.map_res(Role::from_str).parse_complete(i)
}

fn skill(s: &str) -> IResult<Skill> {
    (
        many(1.., recognize(leading_ws(alpha1))),
        leading_ws(preceded(tag("+"), leading_ws(number))),
        opt(parenthesized(terminated(number, leading_ws(tag("ranks"))))),
    )
        .map(|(mut name, value, ranks): (String, _, _)| {
            name.truncate(name.trim_end().len());
            Skill { name, value, ranks }
        })
        .parse_complete(s)
}

fn feature(s: &str) -> IResult<Feature> {
    // next feature should be separated by a double linebreak, if this yields
    // errors, find a better heuristic.
    let end_of_name = s
        .find('.')
        .unwrap_or(usize::MAX)
        .min(s.find("\n\n").unwrap_or(usize::MAX))
        .min(s.len());
    let (s, name) = into(take(end_of_name)).parse_complete(s)?;
    let (s, description) = into(preceded(
        many0(tag(".").or(space1)),
        take_until("\n\n").or(rest),
    ))
    .parse_complete(s)?;
    Ok((s, dbg!(Feature { name, description })))
}

#[test]
fn test_feature() {
    let (rest, feature) = feature("Hello. Test.\n\n").unwrap();
    assert_eq!(rest, "\n\n");
    assert_eq!(feature, Feature {
        name: "Hello".to_string(),
        description: "Test.".to_string()
    });
}

fn action_kind(s: &str) -> IResult<ActionKind> {
    alpha1
        .or(recognize(many1_count(one_of("▷▶↩"))))
        .map_res(ActionKind::from_str)
        .parse_complete(s)
}

fn action(s: &str) -> IResult<Action> {
    // next action should be separated by a double linebreak, if this yields
    // errors, find a better heuristic.
    let (s, kind) = action_kind(s)?;
    let (s, _) = space0(s)?;
    let end_of_name = s
        .find('.')
        .unwrap_or(usize::MAX)
        .min(s.find("\n\n").unwrap_or(usize::MAX))
        .min(s.len());
    let (s, name) = into(take(end_of_name)).parse_complete(s)?;
    let (mut s, mut description): (_, String) = into(preceded(
        many0(tag(".").or(space1)),
        take_until("\n\n").or(rest),
    ))
    .parse_complete(s)?;

    let mut continue_par = false;
    // special handling for bullet lists in actions e.g. in Larkin pages
    continue_par |= description.contains("◆");
    // special handling for page breaks mid action.
    continue_par |= description.ends_with(" ");
    // because of strange stuff between Expert & Socialite
    continue_par &= find_head(s).is_err();
    // ensure the next thing really is not an action
    continue_par &= action_kind(s.trim_start()).is_err();
    if continue_par {
        let (ss, dd) = leading_ws(take_until("\n\n").or(rest)).parse_complete(s)?;
        s = ss;
        description += dd;
    }
    Ok((s, Action {
        kind,
        name,
        description,
    }))
}

fn opportunities_and_complications(s: &str) -> IResult<OpportunityAndComplication> {
    let (s, _) = tag("opportunities and complications").parse_complete(s)?;

    let (s, _ignored_flavor_text) = take_until("\nOpportunity").parse_complete(s)?;
    let (s, opportunity) = leading_ws(preceded(
        (tag("Opportunity"), opt(char('.'))),
        leading_ws(into(take_until("\nComplication"))),
    ))
    .parse_complete(s)?;
    let (s, complication) = leading_ws(preceded(
        (tag("Complication"), opt(char('.'))),
        leading_ws(into(take_until("\n\n"))),
    ))
    .parse_complete(s)?;
    Ok((s, OpportunityAndComplication {
        opportunity,
        complication,
    }))
}

#[test]
fn test_action() {
    let (rest, feature) = action("▶▶ Hello. Test.\n\n").unwrap();
    assert_eq!(rest, "\n\n");
    assert_eq!(feature, Action {
        name: "Hello".to_string(),
        description: "Test.".to_string(),
        kind: ActionKind::Two
    });
}
