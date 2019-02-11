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
#[proc_macro]
pub fn item(input: TokenStream) -> TokenStream {
    let input = input.into_iter();

    let trees = extdot(input);

    TokenStream::from_iter(trees)
}


fn extdot(trees: impl Iterator<Item = TokenTree>) -> impl Iterator<Item = TokenTree> {
    let mut output = vec![];
    let mut last_expression = vec![];
    let mut last_token = None;

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
            (_, TokenTree::Group(ref grp)) => {
                let block = extdot(grp.stream().into_iter());
                let block = TokenStream::from_iter(block);

                TokenTree::Group(Group::new(grp.delimiter(), block))
            }

            _ => token,
        };

        if is_expressionable(&last_token, &token) {
            last_expression.push(token.clone());
        } else {
            output.append(&mut last_expression);
            output.push(token.clone());
        }

        last_token = Some(token);
    }

    output.append(&mut last_expression);
    output.into_iter()
}

fn transliterate(expr: &mut Vec<TokenTree>, grp: &Group) -> TokenTree {
    use std::str::FromStr;

    if grp.stream().is_empty() {
        #[cfg(nightly)]
        grp.span().warning("empty extended dot, consider removing it");

        return TokenTree::Group(Group::new(Delimiter::Brace, grp.stream()));
    }

    let mut output: Vec<TokenTree> = vec![];

    output.extend(TokenStream::from_str("let mut it = ").unwrap());
    output.append(expr);
    output.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));

    // Process group tokens before processing implicits and running replace_it so that recursive
    // usage is handled properly.
    let mut gstream = extdot(grp.stream().into_iter()).into_iter().collect::<Vec<_>>();

    let split_subexprs = |tok: &TokenTree| match tok {
        TokenTree::Punct(ref p) if p.as_char() == ',' => true,
        _ => false,
    };

    for mut subexpr in gstream.split_mut(split_subexprs) {
        if subexpr.is_empty() {
            output.extend(TokenStream::from_str("it").unwrap());
        } else if is_ident(&subexpr) {
            replace_it(subexpr);
            output.extend_from_slice(subexpr);
            output.extend(TokenStream::from_str("(it)").unwrap());
        } else {
            replace_it(&mut subexpr);
            output.extend(subexpr.iter().cloned());
        }

        output.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
    }

    output.pop();

    TokenTree::Group(Group::new(
        Delimiter::Brace,
        TokenStream::from_iter(output.into_iter()),
    ))
}

fn replace_it(block: &mut [TokenTree]) {
    for token in block {
        match token {
            TokenTree::Ident(ref idnt) if idnt.to_string() == "it" => {
                // Replace Span with one that can resolve to extdot's `it`
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

fn is_expressionable(last_token: &Option<TokenTree>, token: &TokenTree) -> bool {
    match token {
        TokenTree::Group(ref grp) if grp.delimiter() == Delimiter::Parenthesis => true,
        TokenTree::Group(ref grp) if grp.delimiter() == Delimiter::Brace => true,
        TokenTree::Group(ref grp) if grp.delimiter() == Delimiter::Bracket => true,
        TokenTree::Ident(_) => true,
        TokenTree::Literal(_) => true,
        TokenTree::Punct(ref punct) if punct.as_char() == '.' => true,
        TokenTree::Punct(ref punct) if punct.as_char() == ':' && punct.spacing() == Spacing::Joint => true,
        TokenTree::Punct(ref punct) if punct.as_char() == ':' && punct.spacing() == Spacing::Alone => {
            match last_token {
                Some(TokenTree::Punct(ref p)) if p.as_char() == ':' && p.spacing() == Spacing::Joint => true,
                _ => false,
            }
        }
        TokenTree::Punct(ref punct) if punct.as_char() == '<' => true,
        TokenTree::Punct(ref punct) if punct.as_char() == '>' => true,
        TokenTree::Punct(ref punct) if punct.as_char() == '?' => true,
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
