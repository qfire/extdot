extern crate proc_macro;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use proc_macro_hack::proc_macro_hack;
use std::iter::FromIterator;

#[proc_macro_hack]
pub fn expr(input: TokenStream) -> TokenStream {
    let input = input.into_iter();

    let trees = extdot(input);

    let output = TokenStream::from_iter(trees);

    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, output)))
}

/// Allows the use of the extended dot notation in expressions.
///
/// # Examples
/// ```rust
///# use extdot_impl as extdot;
/// use std::fmt;
///
/// struct Example {}
///
/// extdot::item!{
///   impl fmt::Display for Example {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///        let v: i32 = -7;
///
///        let v_abs = v.[it.abs()];
///#       assert_eq!(v_abs, 7);
///        let v_pow = v.[it.pow(2)];
///#       assert_eq!(v_pow, 49);
///
///        write!(f, "({}, {})", v_abs, v_pow)
///     }
///   }
/// }
///
/// fn main() {
///     println!("{}", Example {});
/// }
/// ```
#[proc_macro]
pub fn item(input: TokenStream) -> TokenStream {
    let input = input.into_iter();

    let trees = extdot(input);

    TokenStream::from_iter(trees)
}


fn extdot(trees: impl Iterator<Item = TokenTree>) -> impl Iterator<Item = TokenTree> {
    let mut output = vec![];
    let mut last_expression = vec![];

    for token in trees {
        let token = match (last_expression.last(), &token) {
            (Some(TokenTree::Punct(ref punct)), TokenTree::Group(ref grp))
                if punct.as_char() == '.' =>
            {
                // remove last '.' that's part of extdot
                last_expression.pop();

                // re-arrange extended dot syntax into something parsable
                transliterate(&mut last_expression, grp)
            }
            _ => token,
        };

        let token = match token {
            TokenTree::Group(ref grp) => {
                let block = extdot(grp.stream().into_iter());
                let block = TokenStream::from_iter(block);

                TokenTree::Group(Group::new(grp.delimiter(), block))
            }
            _ => token,
        };

        if is_expressionable(&token) {
            last_expression.push(token.clone());
        } else {
            output.append(&mut last_expression);
            output.push(token);
        }
    }

    output.append(&mut last_expression);
    output.into_iter()
}

fn transliterate(expr: &mut Vec<TokenTree>, grp: &Group) -> TokenTree {
    use std::str::FromStr;

    let mut output: Vec<TokenTree> = vec![];

    output.extend(TokenStream::from_str("let mut it = ").unwrap());
    output.append(expr);
    output.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));

    let mut subexpr = grp.stream().into_iter().collect::<Vec<_>>();

    let make_call = is_ident(&subexpr);

    replace_it(&mut subexpr);
    output.extend(subexpr);

    if make_call {
        output.extend(TokenStream::from_str("(it)").unwrap());
    }

    TokenTree::Group(Group::new(
        Delimiter::Brace,
        TokenStream::from_iter(output.into_iter()),
    ))
}

fn replace_it(block: &mut Vec<TokenTree>) {
    for token in block {
        match token {
            TokenTree::Ident(ref idnt) if idnt.to_string() == "it" => {
                // Replace Span with one that can resolve to extdot's it
                *token = TokenTree::Ident(Ident::new("it", Span::call_site()));
            }
            TokenTree::Group(ref grp) => {
                let mut nested = grp.stream().into_iter().collect::<Vec<_>>();

                replace_it(&mut nested);

                *token = TokenTree::Group(Group::new(
                    grp.delimiter(),
                    TokenStream::from_iter(nested.into_iter()),
                ))
            }
            _ => (),
        }
    }
}

// TODO: Currently this doesn't handle struct initializers, need to checking `Spacing` on `:`
fn is_expressionable(token: &TokenTree) -> bool {
    match token {
        TokenTree::Group(ref grp) if grp.delimiter() == Delimiter::Parenthesis => true,
        TokenTree::Group(ref grp) if grp.delimiter() == Delimiter::Brace => true,
        TokenTree::Group(ref grp) if grp.delimiter() == Delimiter::Bracket => true,
        TokenTree::Ident(_) => true,
        TokenTree::Punct(ref punct) if punct.as_char() == '.' => true,
        TokenTree::Punct(ref punct) if punct.as_char() == ':' => true,
        TokenTree::Punct(ref punct) if punct.as_char() == '<' => true,
        TokenTree::Punct(ref punct) if punct.as_char() == '>' => true,
        _ => false,
    }
}

fn is_ident(trees: &[TokenTree]) -> bool {
    for token in trees {
        match token {
            TokenTree::Ident(_) => (),
            TokenTree::Punct(ref punct) if punct.as_char() == ':' => (),
            TokenTree::Literal(_) => return false,
            TokenTree::Group(_) => return false,
            TokenTree::Punct(_) => return false,
        }
    }

    true
}
