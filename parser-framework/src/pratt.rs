use crate::context::ParseContext;
use crate::traits::AstNode;

/// Trait for defining operator precedence and parsing logic for Pratt parsing.
///
/// Pratt parsing (Top-Down Operator Precedence) is an efficient way to parse expressions.
/// Instead of a hierarchy of rules, it uses binding powers to handle precedence.
pub trait PrattConfig<Ctx, Tok, Ast>
where
    Ctx: ParseContext<Tok>,
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    /// Returns the prefix binding power for a token.
    /// Used for unary operators (e.g., -x, !x) or atomic terms (numbers, vars).
    /// Returns `None` if the token cannot start an expression.
    /// The tuple is `((), right_binding_power)`.
    fn prefix_op(&self, token: &Tok) -> Option<((), u8)>;

    /// Returns the infix binding power for a token.
    /// Used for binary operators (e.g., x + y, x * y).
    /// Returns `None` if the token is not an infix operator.
    /// The tuple is `(left_binding_power, right_binding_power)`.
    fn infix_op(&self, token: &Tok) -> Option<(u8, u8)>;

    /// Parses a "null denotation" (prefix or atom).
    /// `token` is the first token (already consumed).
    /// `parser` is a callback to recursively parse an expression with a given minimum binding power.
    fn parse_prefix<F>(&self, token: Tok, ctx: &mut Ctx, parser: &F) -> Option<Ast>
    where
        F: Fn(&mut Ctx, u8) -> Option<Ast>;

    /// Parses a "left denotation" (infix or postfix).
    /// `left` is the expression already parsed on the left.
    /// `token` is the operator token (already consumed).
    /// `r_bp` is the right binding power of the operator.
    /// `parser` is a callback to recursively parse the right-hand side.
    fn parse_infix<F>(
        &self,
        left: Ast,
        token: Tok,
        r_bp: u8,
        ctx: &mut Ctx,
        parser: &F,
    ) -> Option<Ast>
    where
        F: Fn(&mut Ctx, u8) -> Option<Ast>;
}

/// Parses an expression using the Pratt algorithm.
///
/// This function drives the Pratt parsing process using the provided configuration.
pub fn parse_pratt<Ctx, Tok, Ast, Config>(ctx: &mut Ctx, config: &Config, min_bp: u8) -> Option<Ast>
where
    Ctx: ParseContext<Tok>,
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
    Config: PrattConfig<Ctx, Tok, Ast>,
{
    // 1. Consume the first token (prefix or atom)
    let token = ctx.peek().cloned()?;
    ctx.advance();

    // 2. Parse the prefix part (nud)
    // We construct a recursive closure for the callback
    let recursive_parser = |c: &mut Ctx, bp: u8| parse_pratt(c, config, bp);

    let mut left = config.parse_prefix(token, ctx, &recursive_parser)?;

    // 3. Look ahead for an infix operator
    loop {
        // Peek and check binding power without holding the borrow
        let (l_bp, r_bp) = {
            let peek_token = match ctx.peek() {
                Some(t) => t,
                None => break,
            };
            match config.infix_op(peek_token) {
                Some(bp) => bp,
                None => break,
            }
        };

        // 4. Check binding power
        // If the operator binds less tightly than our current context, stop.
        if l_bp < min_bp {
            break;
        }

        // 5. Consume operator and parse infix part (led)
        let op = ctx.advance().unwrap(); // Safe because we peeked

        // Pass right_binding_power to recursive call indirectly via parse_infix
        if let Some(new_left) = config.parse_infix(left.clone(), op, r_bp, ctx, &recursive_parser) {
            left = new_left;
        } else {
            // If infix parse fails, maybe it wasn't an infix usage after all?
            return None;
        }
        continue;
    } // End of loop

    Some(left)
}
