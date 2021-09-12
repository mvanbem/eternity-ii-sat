use std::io::{self, Write};

#[derive(Clone, Copy)]
pub struct Literal(isize);

impl Literal {
    pub fn positive<V>(variable: V) -> Self
    where
        V: Into<usize>,
    {
        Self(variable.into() as isize)
    }

    pub fn negative<V>(variable: V) -> Self
    where
        V: Into<usize>,
    {
        Self(-(variable.into() as isize))
    }
}

pub struct BinaryClause {
    literals: [Literal; 2],
}

impl BinaryClause {
    pub fn new(a: Literal, b: Literal) -> Self {
        Self { literals: [a, b] }
    }

    pub fn print_dimacs_fragment<W: Write>(&self, mut w: W) -> io::Result<()> {
        writeln!(w, "{} {} 0", self.literals[0].0, self.literals[1].0)
    }
}

pub struct LongClause {
    literals: Vec<Literal>,
}

impl LongClause {
    pub fn new(literals: Vec<Literal>) -> Self {
        Self { literals }
    }

    pub fn print_dimacs_fragment<W: Write>(&self, mut w: W) -> io::Result<()> {
        for (i, literal) in self.literals.iter().copied().enumerate() {
            if i > 0 {
                write!(w, " ")?;
            }
            write!(w, "{}", literal.0)?;
        }
        writeln!(w, " 0")
    }
}

#[derive(Default)]
pub struct Clauses {
    binary: Vec<BinaryClause>,
    long: Vec<LongClause>,
}

impl Clauses {
    pub fn push_binary(&mut self, a: Literal, b: Literal) {
        self.binary.push(BinaryClause::new(a, b));
    }

    pub fn push_long(&mut self, literals: Vec<Literal>) {
        self.long.push(LongClause::new(literals));
    }

    pub fn push_unit(&mut self, literal: Literal) {
        self.long.push(LongClause::new(vec![literal]));
    }

    pub fn len(&self) -> usize {
        self.binary.len() + self.long.len()
    }

    pub fn print_dimacs_fragment<W: Write>(&self, mut w: W) -> io::Result<()> {
        for clause in &self.binary {
            clause.print_dimacs_fragment(w.by_ref())?;
        }
        for clause in &self.long {
            clause.print_dimacs_fragment(w.by_ref())?;
        }
        Ok(())
    }

    pub fn emit_at_most_one_of<V>(&mut self, variables: &[V])
    where
        V: Copy + Into<usize>,
    {
        for (i, a) in variables[1..].iter().copied().enumerate() {
            for b in variables[..i].iter().copied() {
                self.push_binary(Literal::negative(a), Literal::negative(b));
            }
        }
    }

    pub fn emit_at_least_one_of<V>(&mut self, variables: &[V])
    where
        V: Copy + Into<usize>,
    {
        self.push_long(
            variables
                .iter()
                .copied()
                .map(|v| Literal::positive(v))
                .collect(),
        );
    }
}
