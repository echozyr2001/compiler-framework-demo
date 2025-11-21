use crate::context::{DefaultContext, ParseContext};
use crate::traits::{AstNode, ParsingRule};
use std::cmp::Reverse;

/// A parser that applies rules in priority order.
/// This is the main orchestrator in the CGP design.
pub struct Parser<Ctx, Tok, Ast>
where
    Ctx: ParseContext<Tok>,
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    context: Ctx,
    rules: Vec<Box<dyn ParsingRule<Ctx, Tok, Ast>>>,
}

impl<Ctx, Tok, Ast> Parser<Ctx, Tok, Ast>
where
    Ctx: ParseContext<Tok>,
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    /// Creates a new parser with the given context and rules.
    pub fn new(context: Ctx, rules: Vec<Box<dyn ParsingRule<Ctx, Tok, Ast>>>) -> Self {
        // Sort rules by priority (highest first)
        let mut sorted_rules = rules;
        sorted_rules.sort_by_key(|rule| Reverse(rule.priority()));

        Self {
            context,
            rules: sorted_rules,
        }
    }

    /// Creates a parser from a token iterator.
    pub fn from_tokens<I>(
        tokens: I,
        rules: Vec<Box<dyn ParsingRule<DefaultContext<Tok>, Tok, Ast>>>,
    ) -> Parser<DefaultContext<Tok>, Tok, Ast>
    where
        I: IntoIterator<Item = Tok>,
    {
        let context = DefaultContext::from_token_iter(tokens);
        Parser::new(context, rules)
    }

    /// Returns a reference to the context.
    pub fn context(&self) -> &Ctx {
        &self.context
    }

    /// Returns a mutable reference to the context.
    pub fn context_mut(&mut self) -> &mut Ctx {
        &mut self.context
    }

    /// Tries to parse the next AST node using the rules.
    ///
    /// This method optimizes rule matching by:
    /// 1. Using quick_check() to skip rules that definitely won't match
    /// 2. Only creating checkpoints when actually trying a rule
    pub fn next_node(&mut self) -> Option<Ast> {
        for rule in &mut self.rules {
            // Quick check: borrow the current token only within this block so the
            // mutable borrow is released before try_parse needs &mut self.context.
            let should_try = {
                let token_ref = self.context.peek();
                !matches!(rule.quick_check(token_ref), Some(false))
            };

            if !should_try {
                continue;
            }

            let checkpoint = self.context.checkpoint();
            if let Some(node) = rule.try_parse(&mut self.context) {
                self.context.commit();
                return Some(node);
            }
            // If rule didn't match, restore context
            self.context.restore(checkpoint);
        }
        None
    }

    /// Parses the entire input and returns all AST nodes.
    ///
    /// This method will continue parsing until EOF is reached or
    /// no progress can be made (indicating a parsing error).
    pub fn parse(&mut self) -> Vec<Ast> {
        let mut nodes = Vec::new();
        while !self.context.is_eof() {
            let offset_before = self.context.token_index();
            if let Some(node) = self.next_node() {
                if self.context.token_index() == offset_before {
                    eprintln!("Warning: No progress made at token index {}", offset_before);
                    break;
                }
                nodes.push(node);
            } else if self.context.token_index() == offset_before {
                eprintln!("Error: No rule matched token at index {}", offset_before);
                if let Some(token) = self.context.peek() {
                    eprintln!("Current token: {:?}", token);
                }
                break;
            }
        }
        nodes
    }

    /// Parses a single AST node and returns it, or None if no rule matches.
    ///
    /// This is a convenience method that calls `next_node()`.
    pub fn parse_one(&mut self) -> Option<Ast> {
        self.next_node()
    }
}
