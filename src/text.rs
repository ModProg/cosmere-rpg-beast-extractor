use std::str::FromStr;

#[cfg(test)]
use insta::assert_debug_snapshot;
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
    fn trim(self) -> impl nom::Parser<&'a str, Output = &'a str, Error = VerboseError<&'a str>>
    where
        O: Into<&'a str>,
    {
        self.map(|i| i.into().trim())
    }

    fn conv<T: From<O>>(
        self,
    ) -> impl nom::Parser<&'a str, Output = T, Error = VerboseError<&'a str>> {
        into(self)
    }
}
impl<'a, P, O> Parser<'a, O> for P where
    P: nom::Parser<&'a str, Output = O, Error = VerboseError<&'a str>>
{
}

pub fn parse_page(out: &str) -> Vec<Beast> {
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
    preceded(
        (tag(name), cut(leading_ws(char(':')))),
        cut(leading_ws(parser)),
    )
    .context(name)
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

fn named_value_with_paren<'a, T, P: Into<String>>(
    name: &'static str,
    value: impl Parser<'a, T>,
    parenthesized_value: impl Parser<'a, P>,
) -> impl Parser<'a, DescValue<T>> {
    named_value(
        name,
        (leading_ws(value), parenthesized(parenthesized_value)),
    )
    .map(|(value, desc)| DescValue {
        value,
        desc: desc.into(),
    })
}

fn beast_with_pretext(s: &str) -> IResult<Beast> {
    let (s, (name, tier, role, size, kind)) = find_head(s)?;
    let (s, (attributes, defenses)) = attributes(s)?;
    let (s, health) = leading_ws(health).parse_complete(s)?;
    // Focus: 3
    let (s, focus) = leading_ws(named_value("Focus", number)).parse_complete(s)?;
    // Investiture: 0
    let (s, investiture) = leading_ws(named_value("investiture", number)).parse_complete(s)?;
    let (s, deflect) = leading_ws(opt(named_value_with_paren(
        "Deflect",
        number,
        take_until(")").trim(),
    )))
    .parse_complete(s)?;
    let (s, movement) = leading_ws(named_value("Movement", movement)).parse_complete(s)?;
    let (s, senses) = leading_ws(named_value_with_paren(
        "Senses",
        number_in_feet,
        take_until(")").trim(),
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
            f.name != "actions"
        }))),
    )))
    .parse_complete(s)?;

    let (s, actions) =
        leading_ws(opt(preceded(tag("actions"), many0(leading_ws(action))))).parse_complete(s)?;

    let (s, (opportunity, complication)) =
        leading_ws(cut(opportunities_and_complications)).parse_complete(s)?;

    Ok((s, Beast {
        name,
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
        opportunity,
        complication,
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
            (leading_ws(into(alpha1)), leading_ws(number_in_feet))
                .map(|(desc, value)| DescValue { value, desc }),
        )),
        opt(parenthesized(into(take_until(")").trim()))),
    )
        .map(|(value, extra, desc)| Movement { value, extra, desc })
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
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
            leading_ws(recognize((digit1, opt(char('*')))).conv()),
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
                physical: physical_defense,
                cognitive: cognitive_defense,
                spiritual: spiritual_defense,
            },
        ),
    ))
}

/// ```text
/// [...]
/// Swordmaster Ardent
/// Tier 1 Rival – Medium Humanoid
/// ```
fn find_head(s: &str) -> IResult<(String, u64, Role, Option<Size>, String)> {
    let (s, (_ignored_pretext, (title, _newline, tier, _space, role))) = many_till(
        anychar.map(|_| ()),
        (into(title.trim()), multispace1, tier, space1, role),
    )
    .context("find beast")
    .parse_complete(s)?;
    let (s, _dash) = (multispace0, one_of("–-"), multispace0)
        .context("dash before size")
        .parse_complete(s)?;
    let (s, size_kind) = opt((
        alpha1.map_res(Size::from_str).context("size"),
        leading_ws(alpha1.conv()).context("kind"),
    ))
    .parse_complete(s)?;
    if let Some((size, kind)) = size_kind {
        Ok((s, (title, tier, role, Some(size), kind)))
    } else {
        let (s, kind) = into(take_until("\n").trim()).parse_complete(s)?;
        Ok((s, (title, tier, role, None, kind)))
    }
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
        leading_ws(recognize((tag("+"), leading_ws(digit1), opt(char('*')))).conv()),
        opt(parenthesized(take_until(")")).conv()),
    )
        .map(|(mut name, value, desc): (String, _, _)| {
            name.truncate(name.trim_end().len());
            Skill { name, value, desc }
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
    let (s, name) = into(take(end_of_name).trim()).parse_complete(s)?;
    let (s, desc) = into(preceded(many0(tag(".").or(space1)), take_until("\n\n").or(rest)).trim())
        .parse_complete(s)?;
    Ok((s, Feature { name, desc }))
}

#[test]
fn test_feature() {
    let (rest, feature) = feature("Hello. Test.\n\n").unwrap();
    assert_eq!(rest, "\n\n");
    assert_eq!(feature, Feature {
        name: "Hello".to_string(),
        desc: "Test.".to_string()
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
    let (s, name) = into(take(end_of_name).trim()).parse_complete(s)?;
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
        desc: description.trim().into(),
    }))
}

fn opportunities_and_complications(s: &str) -> IResult<(Option<String>, Option<String>)> {
    let (s, header) = opt(tag("opportunities and complications")).parse_complete(s)?;
    if header.is_none() {
        return Ok((s, (None, None)));
    }

    let (s, _ignored_flavor_text) = take_until("\nOpportunity").parse_complete(s)?;
    let (s, opportunity) = leading_ws(preceded(
        (tag("Opportunity"), opt(char('.'))),
        leading_ws(into(take_until("\nComplication").trim())),
    ))
    .parse_complete(s)?;
    let (s, complication) = leading_ws(preceded(
        (tag("Complication"), opt(char('.'))),
        leading_ws(into(take_until("\n\n").trim())),
    ))
    .parse_complete(s)?;
    Ok((s, (Some(opportunity), Some(complication))))
}

#[test]
fn test_action() {
    let (rest, feature) = action("▶▶ Hello. Test.\n\n").unwrap();
    assert_eq!(rest, "\n\n");
    assert_eq!(feature, Action {
        name: "Hello".to_string(),
        desc: "Test.".to_string(),
        kind: ActionKind::Two
    });
}

#[test]
fn test_parse() {
    let text = r#" Random Text
Great Fighter
Tier 2 Rival – Medium Humanoid
   Physical   Cognitive   Spiritual
 str def spd int def wil awa def pre
 1 14* 3 0 11 1 3 16 3

Health: 18 (19–36) Focus: 2 Investiture: 3

Movement: 42 ft.
Senses: 21 ft. (sight)
Physical Skills: Walking +4, Heavy Stabbing +3, 
Light Stabbing +4*
Cognitive Skills: Standing +3, Remembering +2, Repairing +2
Spiritual Skills: Seeing +5, Commanding +5, Recognizing +4
Languages: every

features

Tennis player. Gains an advantage on hitting projectiles.

actions

▶ Strike: Racket. Attack +4, reach 5 ft., one target.
Graze: 3 (1d6) blunt damage. Hit: 7 (1d6 + 4) blunt damage.

opportunities and complications

The following options are available when an enemy gains 
an Opportunity or Complication during a scene with the 
Great Fighter:

Opportunity. An enemy can spend #OPPORTUNITY# to prevent the Great
Fighter from being a great fighter, 
until the end of the Great Fighter’s next turn.
Complication. The GM can spend #COMPLICATION# from an enemy’s test to 
have the Great Fighter use their Strike as ↩, 
without spending Focus to do so.

ASdhluadlbasd
    "#;
    assert_debug_snapshot!(parse_page(text), @r#"
    [
        Beast {
            name: "Great Fighter",
            tier: 2,
            role: Rival,
            size: Some(
                Medium,
            ),
            kind: "Humanoid",
            attributes: Attributes {
                strength: "1",
                speed: "3",
                intellect: "0",
                willpower: "1",
                awareness: "3",
                presence: "3",
            },
            defenses: Defenses {
                physical: "14*",
                cognitive: "11",
                spiritual: "16",
            },
            health: Ranged {
                value: 18,
                min: 19,
                max: 36,
            },
            focus: 2,
            investiture: 3,
            deflect: None,
            movement: Movement {
                value: 42,
                extra: [],
                desc: None,
            },
            senses: DescValue {
                value: 21,
                desc: "sight",
            },
            immunities: [],
            physical_skills: [
                Skill {
                    name: "Walking",
                    value: "+4",
                    desc: None,
                },
                Skill {
                    name: "Heavy Stabbing",
                    value: "+3",
                    desc: None,
                },
                Skill {
                    name: "Light Stabbing",
                    value: "+4*",
                    desc: None,
                },
            ],
            cognitive_skills: [
                Skill {
                    name: "Standing",
                    value: "+3",
                    desc: None,
                },
                Skill {
                    name: "Remembering",
                    value: "+2",
                    desc: None,
                },
                Skill {
                    name: "Repairing",
                    value: "+2",
                    desc: None,
                },
            ],
            spiritual_skills: [
                Skill {
                    name: "Seeing",
                    value: "+5",
                    desc: None,
                },
                Skill {
                    name: "Commanding",
                    value: "+5",
                    desc: None,
                },
                Skill {
                    name: "Recognizing",
                    value: "+4",
                    desc: None,
                },
            ],
            surge_skills: [],
            languages: Some(
                [
                    "every",
                ],
            ),
            features: [
                Feature {
                    name: "Tennis player",
                    desc: "Gains an advantage on hitting projectiles.",
                },
            ],
            actions: [
                Action {
                    kind: One,
                    name: "Strike: Racket",
                    desc: "Attack +4, reach 5 ft., one target.\nGraze: 3 (1d6) blunt damage. Hit: 7 (1d6 + 4) blunt damage.",
                },
            ],
            opportunity: Some(
                "An enemy can spend #OPPORTUNITY# to prevent the Great\nFighter from being a great fighter, \nuntil the end of the Great Fighter’s next turn.",
            ),
            complication: Some(
                "The GM can spend #COMPLICATION# from an enemy’s test to \nhave the Great Fighter use their Strike as ↩, \nwithout spending Focus to do so.",
            ),
        },
    ]
    "#)
}
